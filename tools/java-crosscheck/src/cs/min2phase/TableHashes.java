package cs.min2phase;

import java.util.LinkedHashMap;
import java.util.Map;

/** FNV-1a hashes of min2phase's tables after {@code Search.init()}, which are package-private. */
public final class TableHashes {
    private TableHashes() {}

    private static long fnv(long h, long v) {
        return (h ^ (v & 0xffffffffL)) * 0x100000001b3L;
    }

    private static long hash(char[][] t) {
        long h = 0xcbf29ce484222325L;
        for (char[] row : t) for (char v : row) h = fnv(h, v);
        return h;
    }

    private static long hash(int[] t) {
        long h = 0xcbf29ce484222325L;
        for (int v : t) h = fnv(h, v);
        return h;
    }

    private static long hash(char[] t) {
        long h = 0xcbf29ce484222325L;
        for (char v : t) h = fnv(h, v);
        return h;
    }

    public static Map<String, String> all() {
        Search.init();
        Map<String, String> m = new LinkedHashMap<>();
        m.put("UDSliceMove", Long.toHexString(hash(CoordCube.UDSliceMove)));
        m.put("TwistMove", Long.toHexString(hash(CoordCube.TwistMove)));
        m.put("FlipMove", Long.toHexString(hash(CoordCube.FlipMove)));
        m.put("UDSliceConj", Long.toHexString(hash(CoordCube.UDSliceConj)));
        m.put("UDSliceTwistPrun", Long.toHexString(hash(CoordCube.UDSliceTwistPrun)));
        m.put("UDSliceFlipPrun", Long.toHexString(hash(CoordCube.UDSliceFlipPrun)));
        m.put("TwistFlipPrun", Long.toHexString(hash(CoordCube.TwistFlipPrun)));
        m.put("CPermMove", Long.toHexString(hash(CoordCube.CPermMove)));
        m.put("EPermMove", Long.toHexString(hash(CoordCube.EPermMove)));
        m.put("MPermMove", Long.toHexString(hash(CoordCube.MPermMove)));
        m.put("MPermConj", Long.toHexString(hash(CoordCube.MPermConj)));
        m.put("CCombPMove", Long.toHexString(hash(CoordCube.CCombPMove)));
        m.put("CCombPConj", Long.toHexString(hash(CoordCube.CCombPConj)));
        m.put("MCPermPrun", Long.toHexString(hash(CoordCube.MCPermPrun)));
        m.put("EPermCCombPPrun", Long.toHexString(hash(CoordCube.EPermCCombPPrun)));
        m.put("FlipS2RF", Long.toHexString(hash(CubieCube.FlipS2RF)));
        m.put("FlipR2S", Long.toHexString(hash(CubieCube.FlipR2S)));
        m.put("SymStateTwist", Long.toHexString(hash(CubieCube.SymStateTwist)));
        m.put("SymStatePerm", Long.toHexString(hash(CubieCube.SymStatePerm)));
        m.put("Perm2CombP", Long.toHexString(hashBytes(CubieCube.Perm2CombP)));
        return m;
    }

    private static long hashBytes(byte[] t) {
        long h = 0xcbf29ce484222325L;
        for (byte v : t) h = fnv(h, v & 0xff);
        return h;
    }
}
