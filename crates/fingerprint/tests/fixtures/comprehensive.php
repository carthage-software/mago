<?php
/**
 * Comprehensive PHP file with ALL AST nodes
 *
 * @package TestPackage
 * @author Test Author
 * @version 1.0.0
 * @template T of object
 * @template-covariant TValue
 */

declare(strict_types=1, ticks=1);

namespace Foo\Bar\Baz;

use Foo\Bar;
use Foo\Baz as AnotherBaz;

use function strlen;

use const PHP_VERSION;

const GLOBAL_CONST = 'value';

// Class with all features
#[Attribute(\Attribute::TARGET_CLASS)]
#[\SensitiveParameter]
final readonly class TestClass extends ParentClass implements Interface1, Interface2
{
    // Class constant with visibility and attributes
    #[Deprecated]
    public const PUBLIC_CONST = 123;
    private const PRIVATE_CONST = array(1, 2, 3);
    protected const PROTECTED_CONST = [4, 5, 6];

    // Properties with all modifiers
    public int $publicProp = 1;
    protected string $protectedProp;
    private static null|array $privateProp = null;
    public readonly DateTime $readonlyProp;

    // Property with hooks (PHP 8.4)
    public string $hookedProp {
        get => strtoupper($this->publicProp);
        set(string $value) {
            $this->publicProp = strtolower($value);
        }
    }

    // Constructor with property promotion
    public function __construct(
        public string $promoted,
        protected int $promotedProtected = 42,
        private null|array $promotedPrivate = null,
        #[SensitiveParameter]
        string $sensitive = '',
        mixed ...$variadic,
    ) {
        // Variable variables
        $var = 'test';
        $$var = 'dynamic';

        // All assignment operators
        $a = 1;
        $a += 2;
        $a -= 1;
        $a *= 3;
        $a /= 2;
        $a %= 5;
        $a **= 2;
        $a .= 'string';
        $a &= 0xFF;
        $a |= 0x0F;
        $a ^= 0xF0;
        $a <<= 2;
        $a >>= 1;
        $a ??= 'default';

        // All binary operators
        $result = 1 + 2 - (((3 * 4) / 5) % (6 ** 7));
        $result = $a . $var;
        $result = $a == $var;
        $result = $a === $var;
        $result = $a != $var;
        $result = $a !== $var;
        $result = $a <> $var;
        $result = $a < $var;
        $result = $a <= $var;
        $result = $a > $var;
        $result = $a >= $var;
        $result = $a <=> $var;
        $result = $a && $var;
        ($result = $a) and $var;
        $result = $a || $var;
        ($result = $a) or $var;
        ($result = $a) xor $var;
        $result = $a & $var;
        $result = $a | $var;
        $result = $a ^ $var;
        $result = $a << $var;
        $result = $a >> $var;
        $result = $a ?? $var;
        $result = $a instanceof TestClass;

        // All unary operators
        $result = +$a;
        $result = -$a;
        $result = !$a;
        $result = ~$a;
        $result = ++$a;
        $result = --$a;
        $result = $a++;
        $result = $a--;
        $result = @$a;
        $result = (int) $a;
        $result = (bool) $a;
        $result = (float) $a;
        $result = (string) $a;
        $result = (array) $a;
        $result = (object) $a;
        $result = (unset) $a;
        $result = (binary) $a;

        // Reference
        $ref = &$a;

        // Ternary and conditional
        $result = $a ? $var : 'default';
        $result = $a ?: 'default';

        // Array access
        $result = $a[0];
        $result = $a['key'];
        $result = $a[$var];

        // Property/method access
        $result = $this->publicProp;
        $result = $this?->publicProp;
        $result = $this->method();
        $result = $this?->method();
        $result = self::$privateProp;
        $result = static::method();
        $result = parent::method();
        $result = TestClass::PUBLIC_CONST;

        // Function calls
        strlen($var);
        call_user_func($var, $a);

        // Instantiation variants
        $obj = new TestClass('test');
        $obj = new TestClass();
        $obj = new $var();
        $obj = new class {
            public function test()
            {
            }
        };
        $obj = new class() extends ParentClass {
            public function test()
            {
            }
        };

        // Clone
        $clone = clone $obj;

        // Constructs
        isset($var);
        isset($a, $var);
        empty($var);
        eval('echo 1;');
        include 'file.php';
        include_once 'file.php';
        require 'file.php';
        require_once 'file.php';
        print $var;
        exit();
        exit();
        exit(1);
        die();
        die();
        die('message');

        // Magic constants
        $result = __FILE__;
        $result = __DIR__;
        $result = __LINE__;
        $result = __FUNCTION__;
        $result = __CLASS__;
        $result = __TRAIT__;
        $result = __METHOD__;
        $result = __NAMESPACE__;
        $result = __COMPILER_HALT_OFFSET__;

        // Literals
        $result = null;
        $result = true;
        $result = false;
        $result = true;
        $result = false;
        $result = null;
        $result = 123;
        $result = 0x1A;
        $result = 0b1010;
        $result = 0o123;
        $result = 1.23;
        $result = 1.23e10;
        $result = 'string';
        $result = 'string';
        $result = "interpolated $var";
        $result = "interpolated {$var}";
        $result = <<<EOD
        heredoc
        EOD;
        $result = <<<'EOD'
        nowdoc
        EOD;

        // Arrays
        $result = array(1, 2, 3);
        $result = [1, 2, 3];
        $result = array('key' => 'value');
        $result = ['key' => 'value'];
        $result = array(1, ...array(2, 3));
        $result = [1, ...[2, 3]];

        // List
        list($x, $y) = [1, 2];
        [$x, $y] = [1, 2];

        // Arrow function
        $fn = fn($x) => $x * 2;
        $fn = fn($x): int => $x * 2;
        $fn = fn&($x) => $x;
        $fn = static fn($x) => $x;

        // Closure
        $closure = function ($x) {
            return $x;
        };
        $closure = function ($x) use ($var) {
            return $x;
        };
        $closure = function ($x) use (&$var) {
            return $x;
        };
        $closure = function &($x) {
            return $x;
        };
        $closure = static function ($x) {
            return $x;
        };

        // Closure from callable
        $closure = strlen(...);
        $closure = $this->method(...);

        // Match expression
        $result = match ($a) {
            1 => 'one',
            2, 3 => 'two or three',
            default => 'other',
        };

        // Throw expression
        $result = $var ?? throw new Exception('error');
    }

    // Method with all features
    #[Route('/path')]
    final public static function method(
        int $param1,
        string $param2 = 'default',
        null|array $param3 = null,
        mixed ...$rest,
    ): self|static|null {
        // Control flow - if/else
        if ($param1 > 0) {
            echo 'positive';
        } elseif ($param1 < 0) {
            echo 'negative';
        } else {
            echo 'zero';
        }

        // Alternative syntax
        if ($param1):
            echo 'alt';
        elseif ($param1 < 0):
            echo 'alt negative';
        else:
            echo 'alt zero';
        endif;

        // Switch
        switch ($param1) {
            case 1:
                break;
            case 2:
            case 3:
                break;
            default:
                break;
        }

        // Alternative switch
        switch ($param1):
            case 1:
                break;
            default:
                break;
        endswitch;

        // Loops
        while ($param1 > 0) {
            $param1--;
            continue;
        }

        while ($param1 > 0):
            $param1--;
        endwhile;

        do {
            $param1++;
        } while ($param1 < 10);

        for ($i = 0; $i < 10; $i++) {
            if ($i == 5)
                break;
            if ($i == 3)
                continue;
        }

        for ($i = 0; $i < 10; $i++):
        endfor;

        foreach ($param3 as $value) {
            echo $value;
        }

        foreach ($param3 as $key => $value) {
            echo "$key: $value";
        }

        foreach ($param3 as &$value) {
            $value *= 2;
        }

        foreach ($param3 as $key => &$value):
        endforeach;

        // Try-catch-finally
        try {
            throw new Exception('test');
        } catch (Exception $e) {
            echo $e->getMessage();
        } catch (Error|TypeError $e) {
            echo 'error';
        } finally {
            echo 'cleanup';
        }

        // Global/static
        global $globalVar;
        static $staticVar = 1;

        // Echo
        echo 'test';
        echo 'test', 'test2';

        // Return
        return new self();
        return;
    }

    // Abstract method would need abstract class
    // public abstract function abstractMethod(): void;
}

// Interface
interface Interface1
{
    public const INTERFACE_CONST = 1;

    public function interfaceMethod(): void;
}

interface Interface2 extends Interface1
{
}

// Trait
trait TestTrait
{
    public function traitMethod()
    {
    }

    abstract public function abstractTraitMethod();
}

// Class using trait
class TraitUser
{
    use TestTrait;
    use TestTrait, AnotherTrait {
        TestTrait::traitMethod insteadof AnotherTrait;
        AnotherTrait::method as aliasedMethod;
        TestTrait::traitMethod as public publicMethod;
        AnotherTrait::method as protected;
    }

    public function abstractTraitMethod()
    {
    }
}

// Abstract class
abstract class AbstractClass
{
    abstract public function abstractMethod(): void;

    public function concreteMethod()
    {
    }
}

// Enum
enum Status: int
{
    case Pending = 0;
    case Active = 1;
    case Completed = 2;

    public function label(): string
    {
        return match ($this) {
            self::Pending => 'Pending',
            self::Active => 'Active',
            self::Completed => 'Completed',
        };
    }
}

enum SimpleStatus
{
    case Pending;
    case Active;
}

// Standalone function
#[Pure]
function standaloneFunction(int $x, string $y = 'default'): int|string
{
    // Goto
    goto label;

    label:
    echo 'label reached';

    // Yield
    yield $x;
    yield $x => $y;
    yield from [1, 2, 3];

    // Unset
    unset($x, $y);

    // Pipe (if supported)
    // $result = $x |> strtoupper(...) |> strlen(...);

    return $x;
}

// Generator
function generator()
{
    for ($i = 0; $i < 10; $i++) {
        yield $i;
    }
}

// Variadic function
function variadic(...$args)
{
    return $args;
}

// Reference parameters
function byReference(&$param)
{
    $param++;
}

// Declare blocks
declare(ticks=1) {
    // code
}

declare(encoding='UTF-8');

// Namespace blocks
namespace AnotherNamespace {
    const CONST_IN_NAMESPACE = 1;
}

namespace {
    // Global namespace
    const GLOBAL_NAMESPACE_CONST = 1;
}

// Halt compiler
__halt_compiler();
// Everything after this is ignored by PHP
This is raw data
