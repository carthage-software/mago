<?php

namespace Fixture {
    const SOME_CONST = 1;

    /** @pure */
    function pure_function(): int
    {
        return 1;
    }

    /**
     * @pure
     *
     * @throws SomeException
     */
    function pure_function_throws(): int
    {
        return 1;
    }

    /**
     * @pure
     */
    function pure_function_with_by_ref_param(string &$val): int
    {
        $val = 'changed';

        return 1;
    }

    /**
     * @must-use
     */
    function must_use_function(): int
    {
        return 2;
    }

    #[\NoDiscard]
    function no_discard_function(): int
    {
        return 3;
    }

    class SomeClass
    {
        public int $prop = 42;
    }
}

namespace UselessStatements {
    use Fixture\SomeClass;

    use function Fixture\must_use_function;
    use function Fixture\no_discard_function;
    use function Fixture\pure_function;

    /**
     * @mago-expect analysis:unused-statement
     */
    function useless_literal()
    {
        42;
    }

    /**
     * @mago-expect analysis:unused-statement
     */
    function useless_string()
    {
        'hello';
    }

    /**
     * @mago-expect analysis:unused-statement
     */
    function useless_array()
    {
        [1, 2, 3];
    }

    /**
     * @mago-expect analysis:unused-statement
     */
    function useless_variable()
    {
        $a = 1;
        $a;
    }

    /**
     * @mago-expect analysis:unused-statement
     */
    function useless_constant()
    {
        \Fixture\SOME_CONST;
    }

    /**
     * @mago-expect analysis:unused-statement
     */
    function useless_property_access()
    {
        $obj = new SomeClass();
        $obj->prop;
    }

    /**
     * @mago-expect analysis:unused-statement
     */
    function useless_anonymous_class()
    {
        new class {};
    }

    /**
     * @mago-expect analysis:unused-statement
     */
    function useless_closure()
    {
        function () {};
    }

    /**
     * @mago-expect analysis:unused-statement
     */
    function useless_magic_constant()
    {
        __LINE__;
    }

    /**
     * @mago-expect analysis:unused-statement
     */
    function useless_binary_operation()
    {
        1 + 1;
    }

    /**
     * @mago-expect analysis:unused-statement
     */
    function useless_pure_function_call()
    {
        pure_function();
    }

    /**
     * @mago-expect analysis:unused-function-call
     */
    function unused_must_use_function_call()
    {
        must_use_function();
    }

    /**
     * @mago-expect analysis:unused-function-call
     */
    function unused_no_discard_function_call()
    {
        no_discard_function();
    }
}

namespace NotUselessStatements {
    use function Fixture\pure_function_throws;
    use function Fixture\pure_function_with_by_ref_param;

    function used_pure_due_to_throws()
    {
        pure_function_throws();
    }

    function used_pure_due_to_by_ref_param()
    {
        $val = 'initial';
        pure_function_with_by_ref_param($val);
        echo $val;
    }
}
