<?php

declare(strict_types=1);

class Foo {}

class Bar {}

/**
 * @type Services = array{
 *   'foo.service': Foo,
 *   'bar.service': Bar,
 * }
 */
interface GenericServiceLocator
{
    /**
     * @template I of string
     * @param I $service
     * @return $service is key-of<Services> ? Services[I] : object
     */
    public function get(string $service): mixed;
}

/**
 * @type Services = array{
 *   'foo.service': Foo,
 *   'bar.service': Bar,
 * }
 */
interface ConditionalServiceLocator
{
    /**
     * @return $service is key-of<Services> ? Services[$service] : object
     */
    public function get(string $service): mixed;
}

/**
 * @type Services = array{
 *   'foo.service': Foo,
 *   'bar.service': Bar,
 *   ...<string, object>,
 * }
 */
interface OpenServiceLocator
{
    /**
     * @return Services[$service]
     */
    public function get(string $service): mixed;
}

function locate_generic(GenericServiceLocator $locator): object
{
    take_foo($locator->get('foo.service'));
    take_bar($locator->get('bar.service'));

    return $locator->get('baz.service');
}

function locate_conditional(ConditionalServiceLocator $locator): object
{
    take_foo($locator->get('foo.service'));
    take_bar($locator->get('bar.service'));

    return $locator->get('baz.service');
}

function locate_open(OpenServiceLocator $locator): object
{
    take_foo($locator->get('foo.service'));
    take_bar($locator->get('bar.service'));

    return $locator->get('baz.service');
}

function locate_generic_dynamic(GenericServiceLocator $locator, string $service): object
{
    return $locator->get($service);
}

function locate_conditional_dynamic(ConditionalServiceLocator $locator, string $service): object
{
    return $locator->get($service);
}

function locate_open_dynamic(OpenServiceLocator $locator, string $service): object
{
    return $locator->get($service);
}

function take_foo(Foo $_): void {}

function take_bar(Bar $_): void {}
