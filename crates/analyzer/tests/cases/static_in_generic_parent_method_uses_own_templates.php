<?php

declare(strict_types=1);

class Model {}
final class Customer extends Model {}
final class Mechanic extends Model {}

/** @template TModel of Model */
abstract class Factory
{
    final public function __construct() {}

    public static function new(): static
    {
        return new static();
    }

    public function self(): static
    {
        return $this;
    }

    /** @return TModel */
    abstract public function model(): Model;
}

/** @extends Factory<Customer> */
abstract class CustomerFactory extends Factory {}

/**
 * @template TMechanic of Mechanic
 * @extends Factory<TMechanic>
 */
abstract class MechanicFactory extends Factory {}

/** @param Factory<Mechanic> $factory */
function takesMechanicFactory(Factory $factory): void
{
    echo $factory::class;
}

function takesCustomerFactory(CustomerFactory $factory): void
{
    echo $factory::class;
}

function takesMechanic(Mechanic $mechanic): void
{
    echo $mechanic::class;
}

function test(CustomerFactory $customers): void
{
    takesCustomerFactory(CustomerFactory::new());
    takesCustomerFactory($customers->self());
    takesMechanic(MechanicFactory::new()->model());
    takesMechanicFactory(MechanicFactory::new());
}
