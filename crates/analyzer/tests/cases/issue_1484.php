<?php

declare(strict_types=1);

// default before cases — should NOT flag case 0 as unreachable
switch(mt_rand(0, 9)) {
    default: print "any\n"; break;
    case 0: print "0\n"; break;
}

// default after cases — normal order, no issues
switch(mt_rand(0, 9)) {
    case 0: print "0\n"; break;
    default: print "any\n"; break;
}

// default in the middle
switch(mt_rand(0, 9)) {
    case 0: print "0\n"; break;
    default: print "any\n"; break;
    case 1: print "1\n"; break;
}
