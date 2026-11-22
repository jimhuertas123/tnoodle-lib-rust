package org.worldcubeassociation.tnoodle.puzzle;

import java.security.SecureRandom;
import java.util.Random;

import cs.min2phase.SearchWCA;
import cs.min2phase.Tools;
import org.worldcubeassociation.tnoodle.scrambles.AlgorithmBuilder;
import org.worldcubeassociation.tnoodle.scrambles.AlgorithmBuilder.MergingMode;
import org.worldcubeassociation.tnoodle.scrambles.Puzzle;
import org.worldcubeassociation.tnoodle.scrambles.PuzzleStateAndGenerator;

/**
 * tnoodle's seeded 333, 333ni and 333fm scrambles with the search stopped at its
 * first solution ({@code probeMin = 0} instead of THREE_BY_THREE_TIMEMIN), which
 * is what tnoodle-rs does for seeded scrambles (AD-012). Each method is a copy of
 * the tnoodle code it names with only that constant changed; tnoodle's own
 * classes cannot be reused because NoInspection and FewestMoves call
 * ThreeByThreeCubePuzzle.generateRandomMoves through {@code super}.
 */
public final class SeededThreeByThree {
    private SeededThreeByThree() {}

    /** Puzzle.generateSeededScrambles(seed, count). */
    public static String[] scrambles(String shortName, String seed, int count) throws Exception {
        SecureRandom r = Puzzle.getSecureRandom();
        r.setSeed(seed.getBytes());
        ThreeByThreeCubePuzzle puzzle = shortName.equals("333ni") ? new NoInspectionThreeByThreeCubePuzzle()
                : shortName.equals("333fm") ? new ThreeByThreeCubeFewestMovesPuzzle() : new ThreeByThreeCubePuzzle();
        SearchWCA search = new SearchWCA();
        String[] out = new String[count];
        for (int i = 0; i < count; i++) {
            // Puzzle.generateWcaScramble
            PuzzleStateAndGenerator psag;
            do {
                psag = generateRandomMoves(shortName, puzzle, search, r);
            } while (psag.state.solveIn(puzzle.getWcaMinScrambleDistance() - 1) != null);
            out[i] = psag.generator;
        }
        return out;
    }

    private static PuzzleStateAndGenerator generateRandomMoves(String shortName, ThreeByThreeCubePuzzle puzzle,
            SearchWCA search, Random r) throws Exception {
        switch (shortName) {
            case "333":
                return threeByThree(puzzle, search, r, null, null);
            case "333ni": {
                // NoInspectionThreeByThreeCubePuzzle.generateRandomMoves
                CubePuzzle.CubeMove[][] randomOrientationMoves = puzzle.getRandomOrientationMoves(puzzle.size / 2);
                CubePuzzle.CubeMove[] randomOrientation = randomOrientationMoves[r.nextInt(randomOrientationMoves.length)];
                String firstAxisRestriction = randomOrientation.length > 0 ? randomOrientation[0].face.toString() : null;
                PuzzleStateAndGenerator psag = threeByThree(puzzle, search, r, firstAxisRestriction, null);
                return NoInspectionFiveByFiveCubePuzzle.applyOrientation(puzzle, randomOrientation, psag, false);
            }
            case "333fm": {
                // ThreeByThreeCubeFewestMovesPuzzle.generateRandomMoves
                String[] scramblePrefix = AlgorithmBuilder.splitAlgorithm("R' U' F");
                String[] scrambleSuffix = AlgorithmBuilder.splitAlgorithm("R' U' F");
                String solutionLastAxisRestriction = scramblePrefix[scramblePrefix.length - 1].substring(0, 1);
                String solutionFirstAxisRestriction = scrambleSuffix[0].substring(0, 1);
                PuzzleStateAndGenerator psag = threeByThree(puzzle, search, r, solutionFirstAxisRestriction, solutionLastAxisRestriction);
                AlgorithmBuilder ab = new AlgorithmBuilder(puzzle, MergingMode.NO_MERGING);
                ab.appendAlgorithms(scramblePrefix);
                ab.appendAlgorithm(psag.generator);
                ab.appendAlgorithms(scrambleSuffix);
                return ab.getStateAndGenerator();
            }
            default:
                throw new IllegalArgumentException(shortName);
        }
    }

    /** ThreeByThreeCubePuzzle.generateRandomMoves(r, first, last) with probeMin = 0. */
    private static PuzzleStateAndGenerator threeByThree(ThreeByThreeCubePuzzle puzzle, SearchWCA search, Random r,
            String firstAxisRestriction, String lastAxisRestriction) throws Exception {
        String randomState = Tools.randomCube(r);
        String scramble = search.solution(randomState, 21, 60 * 1000, 0, SearchWCA.INVERSE_SOLUTION,
                firstAxisRestriction, lastAxisRestriction).trim();
        AlgorithmBuilder ab = new AlgorithmBuilder(puzzle, MergingMode.CANONICALIZE_MOVES);
        ab.appendAlgorithm(scramble);
        return ab.getStateAndGenerator();
    }
}
