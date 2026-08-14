<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit;

use Mago\Sdk\Analyzer\Metadata\MemberIdentifier;
use Mago\Sdk\Analyzer\ReferenceKind;
use Mago\Sdk\Analyzer\ReferenceOrigin;
use Mago\Sdk\Analyzer\ReferenceRegistry;
use Mago\Sdk\Exception\InvalidArgumentException;
use PHPUnit\Framework\TestCase;

final class ReferenceRegistryTest extends TestCase
{
    public function testReferencesAreCollectedWithoutLosingTheirKinds(): void
    {
        $registry = new ReferenceRegistry();
        $method = new MemberIdentifier('App\Controller', 'index');
        $property = new MemberIdentifier('App\Controller', '$state');
        $overridden = new MemberIdentifier('App\BaseController', 'index');

        $registry->add('Symfony\Kernel', $method);
        $registry->add($method, 'Symfony\Request', ReferenceKind::Signature);
        $file = ReferenceOrigin::file('config/routes.php');
        $registry->add($file, $method);
        $registry->addPropertyRead($method, $property);
        $registry->addPropertyWrite('Doctrine\Hydrator', $property);
        $registry->addOverriddenMember($method, $overridden);
        $registry->addFunctionLikeReturn($method, 'load_controller');

        self::assertSame(
            [
                'Symfony\Kernel',
                $method,
                ReferenceKind::Body,
                $method,
                'Symfony\Request',
                ReferenceKind::Signature,
                $file,
                $method,
                ReferenceKind::Body,
                $method,
                $property,
                ReferenceKind::PropertyRead,
                'Doctrine\Hydrator',
                $property,
                ReferenceKind::PropertyWrite,
                $method,
                $overridden,
                ReferenceKind::OverriddenMember,
                $method,
                'load_controller',
                ReferenceKind::FunctionLikeReturn,
            ],
            $registry->takeReferences(),
        );
        self::assertSame([], $registry->takeReferences());
    }

    public function testEmptySymbolReferenceIsRejected(): void
    {
        $this->expectException(InvalidArgumentException::class);

        (new ReferenceRegistry())->add('', 'App\Controller');
    }

    public function testReferenceOriginsDistinguishSymbolsAndFiles(): void
    {
        $symbol = ReferenceOrigin::symbol(new MemberIdentifier('App\Controller', 'index'));
        $file = ReferenceOrigin::file('src/controller.php');

        self::assertFalse($symbol->isFile());
        self::assertNull($symbol->file);
        self::assertTrue($file->isFile());
        self::assertNull($file->symbol);
    }
}
