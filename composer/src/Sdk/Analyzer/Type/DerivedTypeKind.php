<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
enum DerivedTypeKind
{
    case KeyOf;
    case ValueOf;
    case IntMask;
    case IntMaskOf;
    case PropertiesOf;
    case IndexAccess;
    case New_;
    case TemplateType;
    case Intersection;
}
