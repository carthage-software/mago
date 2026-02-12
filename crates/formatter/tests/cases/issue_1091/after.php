<?php declare(strict_types=1);

final class Cache
{
    /**
     * @return array<class-string<EntityObject>, list<class-string<CacheKeyConfig<mixed>>>>
     */
    private function incorrect(): array
    {
        if (!$this->environment->isProduction()) {
            return $this->provideByEntityClassesCache ??= $this->_provideByEntityClasses();
        }

        return $this->provideByEntityClassesCache ??= $this->cache->get(
            'app.cache_key_registry_by_entity_classes',
            $this->_provideByEntityClasses(...),
        );
    }

    /**
     * @return array<class-string<EntityObject>, list<class-string<CacheKeyConfig<mixed>>>>
     */
    private function ok(): array
    {
        if (!$this->environment->isProduction()) {
            return $this->provideByEntityClassesCache ??= $this->_provideByEntityClasses();
        }

        return $this->provideByEntityClassesCache ??= $this->cache->get(
            'app.cache_key_registry_by_entity_classes',
            $this->_provideByEntityClasses($xx),
        );
    }
}
