<?php declare(strict_types=1);

class IsATest
{
    /**
     * @param string $class
     * @return class-string<ReflectionFunction>|class-string<ReflectionMethod>
     */
    public function is_a_with_allow_string(string $class): string
    {
        if (is_a($class, \ReflectionFunction::class, true)) {
            return $class;
        }

        if (is_a($class, \ReflectionMethod::class, true)) {
            return $class;
        }

        exit('bye');
    }

    /**
     * @param object $obj
     * @return ReflectionFunction
     */
    public function is_a_without_allow_string(object $obj): object
    {
        if (is_a($obj, \ReflectionFunction::class)) {
            return $obj;
        }

        exit('bye');
    }

    /**
     * @param string $class
     * @return class-string<ReflectionFunctionAbstract>
     */
    public function is_subclass_of_with_default_allow_string(string $class): string
    {
        if (is_subclass_of($class, \ReflectionFunctionAbstract::class)) {
            return $class;
        }

        exit('bye');
    }

    /**
     * @param object $obj
     * @return ReflectionFunctionAbstract
     */
    public function is_subclass_of_without_allow_string(object $obj): object
    {
        if (is_subclass_of($obj, \ReflectionFunctionAbstract::class, false)) {
            return $obj;
        }

        exit('bye');
    }
}
