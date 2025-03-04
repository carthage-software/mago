<?php

final readonly class TestCase
{
    private function getStubFileFromConfigType(ConfigType $configType): StubFile
    {
        try {
            $stubPath = dirname(__DIR__) . '/Stubs';

            return match ($configType) {
                ConfigType::CONSOLE => StubFile::from($stubPath . '/console.config.stub.php'),
                ConfigType::CACHE => StubFile::from($stubPath . '/cache.config.stub.php'),
                ConfigType::LOG => StubFile::from($stubPath . '/log.config.stub.php'),
                ConfigType::COMMAND_BUS => StubFile::from($stubPath . '/command-bus.config.stub.php'),
                ConfigType::EVENT_BUS => StubFile::from($stubPath . '/event-bus.config.stub.php'),
                ConfigType::VIEW => StubFile::from($stubPath . '/view.config.stub.php'),
                ConfigType::BLADE => StubFile::from($stubPath . '/blade.config.stub.php'),
                ConfigType::TWIG => StubFile::from($stubPath . '/twig.config.stub.php'),
                ConfigType::DATABASE => StubFile::from($stubPath . '/database.config.stub.php'), // @phpstan-ignore match.alwaysTrue (Because this is a guardrail for the future implementations)
                ConfigType::COMMAND => throw new InvalidArgumentException(sprintf(
                    'The "%s" config type is no longer supported, use ConfigType::COMMAND_BUS instead.',
                    $configType->value,
                )),
                default => throw new InvalidArgumentException(sprintf(
                    'The "%s" config type has no supported stub file.',
                    $configType->value,
                )),
            };
        } catch (InvalidArgumentException $invalidArgumentException) {
            throw new FileGenerationFailedException(sprintf(
                'Cannot retrieve stub file: %s',
                $invalidArgumentException->getMessage(),
            ));
        }
    }
}
