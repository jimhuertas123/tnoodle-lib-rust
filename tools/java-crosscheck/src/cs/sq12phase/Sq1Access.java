package cs.sq12phase;

/** Builds a FullCube from its packed fields, which are package-private. */
public final class Sq1Access {
    private Sq1Access() {}

    public static FullCube cube(int ul, int ur, int dl, int dr, int ml) {
        FullCube c = new FullCube();
        c.ul = ul;
        c.ur = ur;
        c.dl = dl;
        c.dr = dr;
        c.ml = ml;
        return c;
    }
}
