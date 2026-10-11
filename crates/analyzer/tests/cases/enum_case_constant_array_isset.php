<?php

declare(strict_types=1);

enum Locale: string
{
    case DeDe = 'de-DE';
    case FrFr = 'fr-FR';

    public function language(): string
    {
        return substr($this->value, 0, 2);
    }
}

interface HasDefaultLocales
{
    public const DEFAULT_LOCALES = [Locale::DeDe, Locale::FrFr];
}

abstract class BaseResolver implements HasDefaultLocales
{
    protected const MAP = [
        'DE' => Locale::DeDe,
        'FR' => Locale::FrFr,
    ];

    protected const array TYPED_MAP = [
        'DE' => Locale::DeDe,
        'FR' => Locale::FrFr,
    ];
}

final class LanguageResolver extends BaseResolver
{
    public function viaSelf(string $country): ?string
    {
        if (!isset(self::MAP[$country])) {
            return null;
        }

        return self::MAP[$country]->language();
    }

    public function viaStatic(string $country): ?string
    {
        if (!isset(static::MAP[$country])) {
            return null;
        }

        return static::MAP[$country]->language();
    }

    public function viaParent(string $country): ?string
    {
        if (!isset(parent::MAP[$country])) {
            return null;
        }

        return parent::MAP[$country]->language();
    }

    public function viaTypedConstant(string $country): ?string
    {
        if (!isset(self::TYPED_MAP[$country])) {
            return null;
        }

        return self::TYPED_MAP[$country]->language();
    }

    public function viaInterfaceList(int $index): ?string
    {
        if (!isset(self::DEFAULT_LOCALES[$index])) {
            return null;
        }

        return self::DEFAULT_LOCALES[$index]->language();
    }

    public function literalKey(): string
    {
        return self::MAP['DE']->language();
    }
}
