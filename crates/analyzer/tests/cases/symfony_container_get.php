<?php

declare(strict_types=1);

namespace Psr\Container;

/**
 * PSR-11 ContainerInterface stub for testing.
 */
interface ContainerInterface
{
    /**
     * @param string $id
     * @return mixed
     */
    public function get(string $id): mixed;

    public function has(string $id): bool;
}

namespace App\Service;

class Mailer
{
    public function send(): string
    {
        return 'sent';
    }
}

class SmtpTransport
{
    public function connect(): bool
    {
        return true;
    }
}

interface MailerInterface
{
}

namespace App\Tests;

use App\Service\Mailer;
use App\Service\SmtpTransport;
use Psr\Container\ContainerInterface;

function accepts_mailer(Mailer $mailer): void
{
    $mailer->send();
}

function known_string_id_is_resolved(ContainerInterface $container): void
{
    $container->get('app.mailer')->send();

    $mailer = $container->get('app.mailer');
    accepts_mailer($mailer);

    $container->get('app.transport')->connect();
}

function alias_id_resolves_to_the_aliased_service_class(ContainerInterface $container): void
{
    $container->get('App\Service\MailerInterface')->send();
}

// Probe C from PLUGINS-RFC.md: the resolved service exposes its real API,
// so a nonexistent method is a hard error instead of blindness.
function probe_c_nonexistent_method_is_reported(ContainerInterface $container): void
{
    /** @mago-expect analysis:non-existent-method */
    $container->get('app.mailer')->methodThatDoesNotExist();
}

function unknown_id_is_no_opinion(ContainerInterface $container): void
{
    /** @mago-expect analysis:mixed-method-access */
    $container->get('app.unknown')->send();

    /** @mago-expect analysis:mixed-argument */
    accepts_mailer($container->get('app.unknown'));
}

function non_literal_id_is_no_opinion(ContainerInterface $container, string $serviceId): void
{
    /** @mago-expect analysis:mixed-method-access */
    $container->get($serviceId)->send();
}

// The class-string idiom keeps working through the psr-container provider;
// both providers coexist on the same `get` method.
function class_string_idiom_still_resolves(ContainerInterface $container): void
{
    $container->get(SmtpTransport::class)->connect();
}
