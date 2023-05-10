from mygfa import preprocess


def paths_logically_lt(g1, g2):
    """Are the paths in g1 logically "less than" those in g2?
    That is, for all paths p in g1, does the sequence charted by
    p in g1 match the sequence charted by p in g2?
    """
    pathseqs_g1 = preprocess.pathseq(g1)
    pathseqs_g2 = preprocess.pathseq(g2)
    for p in g1.paths.keys():
        if p not in g2.paths.keys() or pathseqs_g1[p] != pathseqs_g2[p]:
            return False
    return True


def logically_lt(g1, g2):
    """Is `g1` logically "less than" `g2`?
    That is, can a user of `g1` use `g2` without a hitch?
    Note that `g2` is allowed to have more stuff than `g1`.

    Will add more line items to this as we think of them!
    """
    return paths_logically_lt(g1, g2)
