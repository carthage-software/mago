<?php declare(strict_types=1);

interface ComponentDefinition {}

interface CoreDefinition extends ComponentDefinition {}

interface ComponentPlugin
{
    /** @return ComponentDefinition[] */
    public function getDefinitions();

    /** @return ComponentDefinition */
    public function getDefinition();
}

interface CorePlugin extends ComponentPlugin
{
    /** @return CoreDefinition[] */
    public function getDefinitions();

    /** @return CoreDefinition */
    public function getDefinition();
}

interface DataProducerPlugin extends CorePlugin {}

// Like Drupal's data producer base class, these overrides should inherit the
// narrowed CorePlugin return types, rather than the ComponentPlugin types.
abstract class DataProducer implements DataProducerPlugin
{
    /** {@inheritdoc} */
    public function getDefinitions()
    {
        /** @var CoreDefinition[] $definitions */
        $definitions = [];
        return $definitions;
    }

    /** {@inheritdoc} */
    public function getDefinition()
    {
        return new class implements CoreDefinition {};
    }
}
