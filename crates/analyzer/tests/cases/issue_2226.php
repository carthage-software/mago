<?php

declare(strict_types=1);

class Terms
{
    public ?int $duration = null;
    public ?Period $period = null;

    public function hasDuration(): bool
    {
        return $this->duration !== null;
    }
}

class Period
{
    public ?int $months = null;
}

class Dto
{
    public ?Terms $terms = null;
}

function take_terms(Terms $_): void {}

function take_period(Period $_): void {}

function take_int(int $_): void {}

function format_duration(Dto $dto): ?string
{
    return $dto->terms?->duration !== null ? 'P' . $dto->terms->duration . 'M' : null;
}

function narrow_not_null(Dto $dto): void
{
    if ($dto->terms?->duration !== null) {
        take_terms($dto->terms);
    }

    if (null !== $dto->terms?->duration) {
        take_terms($dto->terms);
    }
}

function narrow_isset(Dto $dto): void
{
    if (isset($dto->terms?->duration)) {
        take_terms($dto->terms);
    }
}

function narrow_truthy_call(Dto $dto): void
{
    if ($dto->terms?->hasDuration()) {
        take_terms($dto->terms);
    }
}

function narrow_chain(Dto $dto): void
{
    if ($dto->terms?->period?->months !== null) {
        take_terms($dto->terms);
    }

    if ($dto->terms?->period?->months !== null) {
        take_period($dto->terms->period);
    }

    if ($dto->terms?->period?->months !== null) {
        take_int($dto->terms->period->months);
    }
}
