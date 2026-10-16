<?php

declare(strict_types=1);

namespace {
    class A
    {
        public function fooFoo(): void
        {
        }
    }

    class B extends A
    {
        public function barBar(): void
        {
        }
    }

    class C
    {
        public function bazBaz(): void
        {
        }
    }

    function getClassNarrows(A|B|C $value): void
    {
        switch (get_class($value)) {
            case A::class:
                $value->fooFoo();
                break;

            case B::class:
                $value->barBar();
                break;

            case C::class:
                $value->bazBaz();
                break;
        }
    }

    function getClassNarrowsViaVariable(A $value): void
    {
        $class = get_class($value);
        switch ($class) {
            case B::class:
                $value->barBar();
                break;
        }
    }

    function getTypeNarrows(int|string $value): void
    {
        switch (gettype($value)) {
            case "string":
                $value .= '';
                break;

            case "integer":
                $value += 0;
                break;
        }
    }

    function testClassStringSourceInvalidation(A $val, A $replacement): void
    {
        $class = get_class($val);
        $val = $replacement;
        switch ($class) {
            case B::class:
                // @mago-expect analysis:non-existent-method
                $val->barBar();
                break;
        }
    }

    function testClassStringSourceInvalidationIf(A $val, A $replacement): void
    {
        $class = get_class($val);
        $val = $replacement;
        if ($class === B::class) {
            // @mago-expect analysis:non-existent-method
            $val->barBar();
        }
    }
}

namespace ShadowedCase {
    function gettype(mixed $_): string
    {
        return 'string';
    }

    function get_class(object $_): string
    {
        return \B::class;
    }

    function testShadowedGetType(int|string $value): string
    {
        switch (gettype($value)) {
            case "string":
                // @mago-expect analysis:invalid-return-statement
                return $value;
        }

        return '';
    }

    function testShadowedGetTypeIf(int|string $value): string
    {
        if (gettype($value) === 'string') {
            // @mago-expect analysis:invalid-return-statement
            return $value;
        }

        return '';
    }

    function testShadowedGetClass(\A $value): void
    {
        switch (get_class($value)) {
            case \B::class:
                // @mago-expect analysis:non-existent-method
                $value->barBar();
                break;
        }
    }

    function testShadowedGetClassIf(\A $value): void
    {
        if (get_class($value) === \B::class) {
            // @mago-expect analysis:non-existent-method
            $value->barBar();
        }
    }

    function testFullyQualifiedGetType(int|string $value): string
    {
        switch (\gettype($value)) {
            case "string":
                return $value;
        }

        return '';
    }

    function testFullyQualifiedGetTypeIf(int|string $value): string
    {
        if (\gettype($value) === 'string') {
            return $value;
        }

        return '';
    }

    function testFullyQualifiedGetClass(\A|\B $value): void
    {
        switch (\get_class($value)) {
            case \B::class:
                $value->barBar();
                break;
        }
    }

    function testFullyQualifiedGetClassIf(\A|\B $value): void
    {
        if (\get_class($value) === \B::class) {
            $value->barBar();
        }
    }
}

namespace ImportedCase {
    use function ShadowedCase\gettype as shadowed_gettype;
    use function ShadowedCase\get_class as shadowed_get_class;

    function testImportedShadowedGetType(int|string $value): string
    {
        switch (shadowed_gettype($value)) {
            case "string":
                // @mago-expect analysis:invalid-return-statement
                return $value;
        }

        return '';
    }

    function testImportedShadowedGetClass(\A $value): void
    {
        switch (shadowed_get_class($value)) {
            case \B::class:
                // @mago-expect analysis:non-existent-method
                $value->barBar();
                break;
        }
    }
}

namespace StackedCasePreserveUnion {
    final class A { public function onlyA(): void {} }
    final class B {}
    final class C {}

    function example(A|B|C $value): void
    {
        $class = get_class($value);

        switch ($class) {
            case A::class:
            case B::class:
                // @mago-expect analysis:non-existent-method
                $value->onlyA();
                break;
        }
    }
}

namespace UnionComparisonTargetNegation {
    function takesBool(bool $_): void {}

    function example(int|string|bool $value, bool $pickInteger): void
    {
        if (gettype($value) === ($pickInteger ? 'integer' : 'string')) {
            return;
        }

        // @mago-expect analysis:possibly-invalid-argument
        takesBool($value);
    }
}
