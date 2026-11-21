import java.io.BufferedReader;
import java.io.FileReader;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.TreeSet;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import cs.sq12phase.FullCube;
import cs.sq12phase.Sq1Access;
import org.worldcubeassociation.tnoodle.puzzle.CubePuzzle;
import org.worldcubeassociation.tnoodle.puzzle.PyraminxPuzzle;
import org.worldcubeassociation.tnoodle.puzzle.PyraminxSolver;
import org.worldcubeassociation.tnoodle.puzzle.SkewbPuzzle;
import org.worldcubeassociation.tnoodle.puzzle.SkewbSolver;
import org.worldcubeassociation.tnoodle.puzzle.TwoByTwoSolver;
import org.worldcubeassociation.tnoodle.scrambles.AlgorithmBuilder;
import org.worldcubeassociation.tnoodle.scrambles.AlgorithmBuilder.MergingMode;
import org.worldcubeassociation.tnoodle.scrambles.Puzzle;
import org.worldcubeassociation.tnoodle.scrambles.Puzzle.PuzzleState;
import org.worldcubeassociation.tnoodle.scrambles.PuzzleRegistry;

/**
 * Recomputes every case printed by the Rust `crosscheck` example with the
 * original Java tnoodle-lib and reports where the answers differ.
 *
 * Usage: java CrossCheck rust_cases.tsv
 * Exits 1 when any case in a contract check differs.
 */
public class CrossCheck {
    private static final String NONE = "-";
    private static final Pattern FILL = Pattern.compile("fill[=:]\"?\\s*(#[0-9a-fA-F]{3,6})");
    private static final Pattern RECT = Pattern.compile("<rect\\s([^>]*)>");

    /** Per check: cases seen, cases that differ, and the first few differences. */
    private static final Map<String, int[]> counts = new LinkedHashMap<>();
    private static final Map<String, List<String>> examples = new LinkedHashMap<>();
    /** Checks with an intentional difference, read from the `Check` column of PARITY.md. */
    private static final java.util.Set<String> expected = new java.util.HashSet<>();
    private static final Pattern PARITY_ROW = Pattern.compile("^\\|\\s*`([a-z0-9-]+)`\\s*\\|");

    private static final Map<String, List<Integer>> rustLengths = new LinkedHashMap<>();
    private static final Map<String, TreeSet<String>> rustTokens = new LinkedHashMap<>();

    private static void record(String check, boolean same, String detail) {
        int[] c = counts.computeIfAbsent(check, k -> new int[2]);
        c[0]++;
        if (!same) {
            c[1]++;
            List<String> ex = examples.computeIfAbsent(check, k -> new ArrayList<>());
            if (ex.size() < 3) {
                ex.add(detail);
            }
        }
    }

    /** A fresh {@code SHA1PRNG} seeded with the hex-encoded bytes, as tnoodle seeds it. */
    private static java.security.SecureRandom sha1prng(String hexSeed) throws Exception {
        byte[] seed = new byte[hexSeed.length() / 2];
        for (int i = 0; i < seed.length; i++) {
            seed[i] = (byte) Integer.parseInt(hexSeed.substring(2 * i, 2 * i + 2), 16);
        }
        java.security.SecureRandom r = java.security.SecureRandom.getInstance("SHA1PRNG", "SUN");
        r.setSeed(seed);
        return r;
    }

    private static String orNone(String s) {
        return s == null ? NONE : s.trim();
    }

    private static int moveCount(String alg) {
        alg = alg.trim();
        return alg.isEmpty() || alg.equals(NONE) ? 0 : alg.split("\\s+").length;
    }

    private static String canonicalize(Puzzle puzzle, String alg) {
        try {
            AlgorithmBuilder ab = new AlgorithmBuilder(puzzle, MergingMode.CANONICALIZE_MOVES);
            ab.appendAlgorithm(alg);
            return ab.toString();
        } catch (org.worldcubeassociation.tnoodle.scrambles.InvalidMoveException e) {
            return "ERR";
        } catch (Throwable t) {
            return "PANIC";
        }
    }

    /** Sticker colour by position ("x,y") from the rects of a cube SVG. */
    private static Map<String, String> stickers(String svg) {
        Map<String, String> out = new java.util.TreeMap<>();
        Matcher rect = RECT.matcher(svg);
        while (rect.find()) {
            String attrs = rect.group(1);
            Matcher x = attr("x").matcher(attrs), y = attr("y").matcher(attrs), fill = FILL.matcher(attrs);
            if (x.find() && y.find() && fill.find()) {
                String key = Math.round(Double.parseDouble(x.group(1))) + "," + Math.round(Double.parseDouble(y.group(1)));
                out.put(key, fill.group(1).toLowerCase());
            }
        }
        return out;
    }

    private static Pattern attr(String name) {
        return Pattern.compile("\\b" + name + "=\"([-0-9.]+)\"");
    }

    /** Rust colour to Java colour, per cube size, learned from the solved renders. */
    private static final Map<Integer, Map<String, String>> colourMap = new java.util.HashMap<>();

    private static String javaSvg(CubePuzzle puzzle, String alg) {
        try {
            return puzzle.drawScramble(alg, null).toString();
        } catch (Throwable t) {
            return "";
        }
    }

    private static Puzzle registry(String shortName) {
        for (PuzzleRegistry p : PuzzleRegistry.values()) {
            if (p.getScrambler().getShortName().equals(shortName)) {
                return p.getScrambler();
            }
        }
        throw new IllegalArgumentException(shortName);
    }

    private static boolean isCube(String shortName) {
        return shortName.matches("[2-7]{3}(fast|ni|fm)?");
    }

    public static void main(String[] args) throws Exception {
        if (args.length > 1) {
            for (String row : java.nio.file.Files.readAllLines(java.nio.file.Paths.get(args[1]))) {
                Matcher m = PARITY_ROW.matcher(row);
                if (m.find()) {
                    expected.add(m.group(1));
                }
            }
        }
        CubePuzzle[] cubes = new CubePuzzle[8];
        for (int size = 2; size <= 7; size++) {
            cubes[size] = new CubePuzzle(size);
        }
        TwoByTwoSolver twoSolver = new TwoByTwoSolver();
        SkewbSolver skewbSolver = new SkewbSolver();
        PyraminxSolver pyraSolver = new PyraminxSolver();
        SkewbPuzzle skewb = new SkewbPuzzle();
        PyraminxPuzzle pyra = new PyraminxPuzzle();
        cs.min2phase.Search twoPhase = new cs.min2phase.Search();

        try (BufferedReader in = new BufferedReader(new FileReader(args[0]))) {
            String line;
            while ((line = in.readLine()) != null) {
                String[] f = line.split("\t", -1);
                switch (f[0]) {
                    case "jr-bounded": {
                        java.security.SecureRandom r = sha1prng(f[1]);
                        int bound = Integer.parseInt(f[2]);
                        StringBuilder java = new StringBuilder();
                        for (int i = 0; i < 1000; i++) {
                            java.append(i == 0 ? "" : ",").append(r.nextInt(bound));
                        }
                        record("javarandom-next-int-bounded", java.toString().equals(f[3]),
                                "seed " + f[1] + " bound " + bound);
                        break;
                    }
                    case "jr-other": {
                        java.security.SecureRandom r = sha1prng(f[1]);
                        StringBuilder java = new StringBuilder();
                        for (int i = 0; i < 1000; i++) {
                            java.append(i == 0 ? "" : ",");
                            switch (f[2]) {
                                case "int": java.append(r.nextInt()); break;
                                case "long": java.append(r.nextLong()); break;
                                case "boolean": java.append(r.nextBoolean()); break;
                                case "double": java.append(String.format("%016x", Double.doubleToLongBits(r.nextDouble()))); break;
                                case "bytes": {
                                    byte[] b = new byte[37];
                                    r.nextBytes(b);
                                    for (byte x : b) java.append(String.format("%02x", x));
                                    break;
                                }
                                default: throw new IllegalArgumentException(f[2]);
                            }
                        }
                        record("javarandom-other-methods", java.toString().equals(f[3]),
                                "seed " + f[1] + " method " + f[2]);
                        break;
                    }
                    case "canon":
                    case "canonrot": {
                        int size = Integer.parseInt(f[1]);
                        String java = canonicalize(cubes[size], f[2]);
                        record(f[0], java.equals(f[3]),
                                size + "x" + size + " " + f[2] + "\n      java: " + java + "\n      rust: " + f[3]);
                        break;
                    }
                    case "svg":
                    case "svgrot": {
                        int size = Integer.parseInt(f[1]);
                        Map<String, String> java = stickers(javaSvg(cubes[size], f[2]));
                        Map<String, String> rust = stickers(f[3].replace("\\n", "\n"));
                        if (f[2].isEmpty()) {
                            // Same sticker positions, and each Rust colour stands for one Java colour.
                            Map<String, String> map = new java.util.HashMap<>();
                            boolean consistent = java.keySet().equals(rust.keySet());
                            for (String pos : rust.keySet()) {
                                String prev = map.put(rust.get(pos), java.get(pos));
                                consistent &= prev == null || prev.equals(java.get(pos));
                            }
                            consistent &= new java.util.HashSet<>(map.values()).size() == map.size();
                            colourMap.put(size, map);
                            record("svg-solved-layout", consistent, size + "x" + size + " java " + java + "\n      rust " + rust);
                            break;
                        }
                        Map<String, String> translated = new java.util.TreeMap<>();
                        Map<String, String> map = colourMap.get(size);
                        rust.forEach((pos, colour) -> translated.put(pos, map.getOrDefault(colour, colour)));
                        record(f[0], java.equals(translated),
                                size + "x" + size + " " + f[2] + "\n      java: " + java + "\n      rust: " + translated);
                        break;
                    }
                    case "222": {
                        TwoByTwoSolver.TwoByTwoState s = new TwoByTwoSolver.TwoByTwoState();
                        s.permutation = Integer.parseInt(f[1]);
                        s.orientation = Integer.parseInt(f[2]);
                        String solve = orNone(twoSolver.solveIn(s, 11));
                        String exact = orNone(twoSolver.generateExactly(s, 11));
                        record("222-solve", solve.equals(f[3]), line + "\n      java: " + solve);
                        record("222-generate", exact.equals(f[4]), line + "\n      java: " + exact);
                        boolean reaches;
                        try {
                            TwoByTwoSolver.TwoByTwoState got = ((CubePuzzle.CubeState) cubes[2].getSolvedState()
                                    .applyAlgorithm(f[4])).toTwoByTwoState();
                            reaches = moveCount(f[4]) == 11 && got.permutation == s.permutation
                                    && got.orientation == s.orientation;
                        } catch (Exception e) {
                            reaches = false;
                        }
                        record("222-generate-reaches-state", reaches, line);
                        break;
                    }
                    case "skewb": {
                        SkewbSolver.SkewbSolverState s = new SkewbSolver.SkewbSolverState();
                        s.perm = Integer.parseInt(f[1]);
                        s.twst = Integer.parseInt(f[2]);
                        // Java picks among optimal solutions at random, so compare
                        // the optimal length and check Rust's moves on Java's model.
                        String solve = orNone(skewbSolver.solveIn(s, 11));
                        record("skewb-optimal-length", moveCount(solve) == moveCount(f[3]),
                                line + "\n      java: " + solve);
                        boolean reaches;
                        boolean solves;
                        try {
                            PuzzleState state = skewb.getSolvedState().applyAlgorithm(f[4]);
                            SkewbSolver.SkewbSolverState got = ((SkewbPuzzle.SkewbState) state).toSkewbSolverState();
                            reaches = moveCount(f[4]) == 11 && got.perm == s.perm && got.twst == s.twst;
                            solves = state.applyAlgorithm(f[3]).isSolved();
                        } catch (Exception e) {
                            reaches = false;
                            solves = false;
                        }
                        record("skewb-generate-reaches-state", reaches, line);
                        record("skewb-solve-solves", solves, line);
                        break;
                    }
                    case "pyram": {
                        PyraminxSolver.PyraminxSolverState s = new PyraminxSolver.PyraminxSolverState();
                        s.edgePerm = Integer.parseInt(f[1]);
                        s.edgeOrient = Integer.parseInt(f[2]);
                        s.cornerOrient = Integer.parseInt(f[3]);
                        s.tips = Integer.parseInt(f[4]);
                        String solve = orNone(pyraSolver.solveIn(s, 20, false));
                        record("pyram-optimal-length", moveCount(solve) == moveCount(f[5]),
                                line + "\n      java: " + solve);
                        boolean reaches;
                        boolean solves;
                        try {
                            PuzzleState state = pyra.getSolvedState().applyAlgorithm(f[6]);
                            PyraminxSolver.PyraminxSolverState got =
                                    ((PyraminxPuzzle.PyraminxState) state).toPyraminxSolverState();
                            reaches = moveCount(f[6]) == 11 + s.unsolvedTips()
                                    && got.edgePerm == s.edgePerm && got.edgeOrient == s.edgeOrient
                                    && got.cornerOrient == s.cornerOrient && got.tips == s.tips;
                            solves = state.applyAlgorithm(f[5]).isSolved();
                        } catch (Exception e) {
                            reaches = false;
                            solves = false;
                        }
                        record("pyram-generate-reaches-state", reaches, line);
                        record("pyram-solve-solves", solves, line);
                        break;
                    }
                    case "sq1": {
                        int[] v = new int[5];
                        for (int i = 0; i < 5; i++) {
                            v[i] = Integer.parseInt(f[1 + i]);
                        }
                        FullCube c1 = Sq1Access.cube(v[0], v[1], v[2], v[3], v[4]);
                        FullCube c2 = Sq1Access.cube(v[0], v[1], v[2], v[3], v[4]);
                        String solution = orNone(new cs.sq12phase.Search().solution(c1, cs.sq12phase.Search.INVERSE_SOLUTION));
                        String optimal = orNone(new cs.sq12phase.Search().solutionOpt(c2, 10, cs.sq12phase.Search.INVERSE_SOLUTION));
                        // Both sides end with a separator space; the app trims it.
                        record("sq1-solution", solution.equals(f[6].trim()), line + "\n      java: " + solution);
                        record("sq1-solution-opt", optimal.equals(f[7].trim()), line + "\n      java: " + optimal);
                        break;
                    }
                    case "333solve": {
                        String facelets = f[1];
                        String rust = f[2];
                        boolean valid = !rust.startsWith("ERR")
                                && cs.min2phase.Tools.fromScramble(rust).equals(facelets)
                                && moveCount(rust) <= 21;
                        record("333solve-reaches-state", valid, line);
                        String java = twoPhase.solution(facelets, 21, 1000000, 0, cs.min2phase.Search.INVERSE_SOLUTION).trim();
                        record("333solve-same-as-java", java.equals(rust), line + "\n      java: " + java);
                        break;
                    }
                    case "scramble": {
                        String shortName = f[1];
                        String scramble = f[2].replace("\\n", "\n");
                        Puzzle puzzle = registry(shortName);
                        String detail = shortName + ": " + scramble;
                        PuzzleState state;
                        try {
                            state = puzzle.getSolvedState().applyAlgorithm(scramble);
                        } catch (Exception e) {
                            record("scramble-valid", false, detail + " (" + e.getMessage() + ")");
                            break;
                        }
                        record("scramble-valid", true, detail);
                        record("scramble-not-too-close",
                                state.solveIn(puzzle.getWcaMinScrambleDistance() - 1) == null, detail);
                        if (isCube(shortName)) {
                            // NI scrambles end with a random orientation that tnoodle
                            // appends after canonicalizing: Rw/Uw/Fw turns on 3x3, x/y/z
                            // rotations on 4x4, and 3Rw/3Uw/3Fw turns on 5x5.
                            String suffix = shortName.equals("333ni") ? "[RUF]w"
                                    : shortName.equals("444ni") ? "[xyz]"
                                    : shortName.equals("555ni") ? "3[RUF]w" : "(?!)";
                            String moves = scramble.replaceAll("(\\s+" + suffix + "[2']?)+$", "");
                            record("scramble-canonical", canonicalize(puzzle, moves).equals(moves), detail);
                        }
                        rustLengths.computeIfAbsent(shortName, k -> new ArrayList<>()).add(moveCount(scramble));
                        rustTokens.computeIfAbsent(shortName, k -> new TreeSet<>())
                                .addAll(Arrays.asList(scramble.trim().split("\\s+")));
                        break;
                    }
                    default:
                        throw new IllegalArgumentException("unknown case: " + f[0]);
                }
            }
        }

        boolean failed = false;
        System.out.println("check                           cases  differ");
        for (Map.Entry<String, int[]> e : counts.entrySet()) {
            boolean listed = expected.contains(e.getKey());
            String status = e.getValue()[1] == 0 ? "OK" : listed ? "expected" : "FAIL";
            failed |= e.getValue()[1] != 0 && !listed;
            System.out.printf("%-30s %6d %7d  %s%n", e.getKey(), e.getValue()[0], e.getValue()[1], status);
        }

        // Lengths and move tokens next to what Java itself generates.
        System.out.println();
        System.out.println("puzzle    rust min/avg/max      java min/avg/max      moves Java never uses");
        Random r = new Random(1);
        for (Map.Entry<String, List<Integer>> e : rustLengths.entrySet()) {
            Puzzle puzzle = registry(e.getKey());
            List<Integer> java = new ArrayList<>();
            TreeSet<String> javaTokens = new TreeSet<>();
            for (int i = 0; i < 4 * e.getValue().size(); i++) {
                String s = puzzle.generateWcaScramble(r);
                java.add(moveCount(s));
                javaTokens.addAll(Arrays.asList(s.trim().split("\\s+")));
            }
            TreeSet<String> extra = new TreeSet<>(rustTokens.get(e.getKey()));
            extra.removeAll(javaTokens);
            System.out.printf("%-9s %-21s %-21s %s%n", e.getKey(), stats(e.getValue()), stats(java),
                    extra.isEmpty() ? "-" : extra);
        }

        for (Map.Entry<String, List<String>> e : examples.entrySet()) {
            System.out.println();
            System.out.println("first differences in " + e.getKey() + ":");
            for (String ex : e.getValue()) {
                System.out.println("  - " + ex);
            }
        }
        System.exit(failed ? 1 : 0);
    }

    private static String stats(List<Integer> xs) {
        int min = Integer.MAX_VALUE, max = 0;
        double sum = 0;
        for (int x : xs) {
            min = Math.min(min, x);
            max = Math.max(max, x);
            sum += x;
        }
        return String.format("%d/%.1f/%d", min, sum / xs.size(), max);
    }
}
