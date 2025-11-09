<?php

// Test 1: Final constant protection
class ParentWithFinalConstant {
    final public const string FINAL_CONST = 'parent';
}

class ChildOverridesFinal extends ParentWithFinalConstant {
    // @mago-expect analysis:override-final-constant
    public const string FINAL_CONST = 'child';
}

// Test 2: Visibility widening violation (public -> protected)
class ParentPublicConstant {
    public const VALUE = 1;
}

class ChildNarrowsToProtected extends ParentPublicConstant {
    // @mago-expect analysis:incompatible-constant-access
    protected const VALUE = 1;
}

// Test 3: Visibility widening violation (public -> private)
class ParentPublicConstant2 {
    public const VALUE = 2;
}

class ChildNarrowsToPrivate extends ParentPublicConstant2 {
    // @mago-expect analysis:incompatible-constant-access
    private const VALUE = 2;
}

// Test 4: Valid same visibility (public -> public)
class ParentSameVisibility {
    public const VALUE = 100;
}

class ChildSameVisibility extends ParentSameVisibility {
    public const VALUE = 200;
}
