test_inference! {
    name = empty_construct,
    cases = {
        "<?php empty(0);" => "true",
        "<?php empty(1);" => "false",
        "<?php empty([]);" => "true",
        "<?php empty([1]);" => "false",
        "<?php $x = 0; empty($x);" => "true",
        "<?php $x = 'a'; empty($x);" => "false",
    }
}

test_inference! {
    name = isset_construct,
    cases = {
        "<?php $a = 1; isset($a);" => "true",
        "<?php $a = null; isset($a);" => "false",
        "<?php $a = 1; $b = 2; isset($a, $b);" => "true",
        "<?php isset($undefined);" => "bool",
    }
}

test_inference! {
    name = print_eval_include,
    cases = {
        "<?php print 'x';" => "int(1)",
        "<?php eval('code');" => "mixed",
        "<?php include 'f.php';" => "mixed",
        "<?php require 'f.php';" => "mixed",
        "<?php require_once 'f.php';" => "mixed",
    }
}

test_inference! {
    name = exit_construct,
    cases = {
        "<?php exit;" => "never",
        "<?php exit(1);" => "never",
        "<?php die('bye');" => "never",
    }
}
