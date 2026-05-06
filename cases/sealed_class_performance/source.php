<?php

declare(strict_types=1);

/**
 * @psalm-inheritors BYD|BMW|Mercedes|Toyota|Honda|Ford|Chevrolet|Nissan|Volkswagen|Audi|Porsche|Tesla|Ferrari|Lamborghini|Mazda|Subaru|Hyundai|Kia|Volvo|Jaguar
 */
abstract class CarMaker
{
    abstract public function getAwesomeCar(): string;
}

class BYD extends CarMaker
{
    public function u9(): string
    {
        return 'BYD - Yangwang U9';
    }

    public function getAwesomeCar(): string
    {
        return $this->u9();
    }
}

class BMW extends CarMaker
{
    public function i8(): string
    {
        return 'BMW - i8';
    }

    public function getAwesomeCar(): string
    {
        return $this->i8();
    }
}

class Mercedes extends CarMaker
{
    public function sClass(): string
    {
        return 'Mercedes - S-Class';
    }

    public function getAwesomeCar(): string
    {
        return $this->sClass();
    }
}

class Toyota extends CarMaker
{
    public function supra(): string
    {
        return 'Toyota - Supra';
    }

    public function getAwesomeCar(): string
    {
        return $this->supra();
    }
}

class Honda extends CarMaker
{
    public function civic(): string
    {
        return 'Honda - Civic Type R';
    }

    public function getAwesomeCar(): string
    {
        return $this->civic();
    }
}

class Ford extends CarMaker
{
    public function mustang(): string
    {
        return 'Ford - Mustang';
    }

    public function getAwesomeCar(): string
    {
        return $this->mustang();
    }
}

class Chevrolet extends CarMaker
{
    public function corvette(): string
    {
        return 'Chevrolet - Corvette';
    }

    public function getAwesomeCar(): string
    {
        return $this->corvette();
    }
}

class Nissan extends CarMaker
{
    public function gtr(): string
    {
        return 'Nissan - GT-R';
    }

    public function getAwesomeCar(): string
    {
        return $this->gtr();
    }
}

class Volkswagen extends CarMaker
{
    public function golf(): string
    {
        return 'Volkswagen - Golf GTI';
    }

    public function getAwesomeCar(): string
    {
        return $this->golf();
    }
}

class Audi extends CarMaker
{
    public function r8(): string
    {
        return 'Audi - R8';
    }

    public function getAwesomeCar(): string
    {
        return $this->r8();
    }
}

class Porsche extends CarMaker
{
    public function gt3(): string
    {
        return 'Porsche - 911 GT3';
    }

    public function getAwesomeCar(): string
    {
        return $this->gt3();
    }
}

class Tesla extends CarMaker
{
    public function modelS(): string
    {
        return 'Tesla - Model S Plaid';
    }

    public function getAwesomeCar(): string
    {
        return $this->modelS();
    }
}

class Ferrari extends CarMaker
{
    public function laFerrari(): string
    {
        return 'Ferrari - LaFerrari';
    }

    public function getAwesomeCar(): string
    {
        return $this->laFerrari();
    }
}

class Lamborghini extends CarMaker
{
    public function aventador(): string
    {
        return 'Lamborghini - Aventador';
    }

    public function getAwesomeCar(): string
    {
        return $this->aventador();
    }
}

class Mazda extends CarMaker
{
    public function rx7(): string
    {
        return 'Mazda - RX-7';
    }

    public function getAwesomeCar(): string
    {
        return $this->rx7();
    }
}

class Subaru extends CarMaker
{
    public function wrx(): string
    {
        return 'Subaru - WRX STI';
    }

    public function getAwesomeCar(): string
    {
        return $this->wrx();
    }
}

class Hyundai extends CarMaker
{
    public function n(): string
    {
        return 'Hyundai - N Series';
    }

    public function getAwesomeCar(): string
    {
        return $this->n();
    }
}

class Kia extends CarMaker
{
    public function stinger(): string
    {
        return 'Kia - Stinger GT';
    }

    public function getAwesomeCar(): string
    {
        return $this->stinger();
    }
}

class Volvo extends CarMaker
{
    public function polestar(): string
    {
        return 'Volvo - Polestar 1';
    }

    public function getAwesomeCar(): string
    {
        return $this->polestar();
    }
}

class Jaguar extends CarMaker
{
    public function fType(): string
    {
        return 'Jaguar - F-Type';
    }

    public function getAwesomeCar(): string
    {
        return $this->fType();
    }
}

function cool_car(CarMaker $maker): string
{
    if ($maker instanceof BYD) {
        return $maker->u9();
    }

    if ($maker instanceof BMW) {
        return $maker->i8();
    }

    if ($maker instanceof Mercedes) {
        return $maker->sClass();
    }

    if ($maker instanceof Toyota) {
        return $maker->supra();
    }

    if ($maker instanceof Honda) {
        return $maker->civic();
    }

    if ($maker instanceof Ford) {
        return $maker->mustang();
    }

    if ($maker instanceof Chevrolet) {
        return $maker->corvette();
    }

    if ($maker instanceof Nissan) {
        return $maker->gtr();
    }

    if ($maker instanceof Volkswagen) {
        return $maker->golf();
    }

    if ($maker instanceof Audi) {
        return $maker->r8();
    }

    if ($maker instanceof Porsche) {
        return $maker->gt3();
    }

    if ($maker instanceof Tesla) {
        return $maker->modelS();
    }

    if ($maker instanceof Ferrari) {
        return $maker->laFerrari();
    }

    if ($maker instanceof Lamborghini) {
        return $maker->aventador();
    }

    if ($maker instanceof Mazda) {
        return $maker->rx7();
    }

    if ($maker instanceof Subaru) {
        return $maker->wrx();
    }

    if ($maker instanceof Hyundai) {
        return $maker->n();
    }

    if ($maker instanceof Kia) {
        return $maker->stinger();
    }

    if ($maker instanceof Volvo) {
        return $maker->polestar();
    }

    return $maker->getAwesomeCar();
}
