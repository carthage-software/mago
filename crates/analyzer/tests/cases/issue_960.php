<?php

abstract class Constraint
{
    public const CLASS_CONSTRAINT = 'class';
    public const PROPERTY_CONSTRAINT = 'property';

    /**
     * @return self::CLASS_CONSTRAINT|self::PROPERTY_CONSTRAINT|array<self::CLASS_CONSTRAINT|self::PROPERTY_CONSTRAINT>
     */
    public function getTargets(): string|array
    {
        return self::PROPERTY_CONSTRAINT;
    }
}

#[Attribute(Attribute::TARGET_CLASS)]
class ValidTaxes extends Constraint
{
    public function getTargets(): string
    {
        return self::CLASS_CONSTRAINT;
    }
}
