<?php

declare(strict_types=1);

namespace NewUtility {
    class Foo {}
    class Bar {}
    class Baz {}

    function accept_foo(Foo $_): void {}
    function accept_bar(Bar $_): void {}
    function accept_baz(Baz $_): void {}
    function accept_object(object $_): void {}

    /** @return new<Foo::class> */
    function make_from_class_const(): Foo
    {
        return new Foo();
    }

    /** @return new<class-string<Bar>> */
    function make_from_bounded_class_string(): Bar
    {
        return new Bar();
    }

    /** @return new<class-string> */
    function make_from_unbounded_class_string(): object
    {
        return new Baz();
    }

    function assertions(): void
    {
        accept_foo(make_from_class_const());
        accept_bar(make_from_bounded_class_string());
        accept_object(make_from_unbounded_class_string());
    }
}

namespace TemplateTypeSingle {
    class Item {}
    class SubItem {}

    function accept_item(Item $_): void {}
    function accept_subitem(SubItem $_): void {}

    /**
     * @template T
     */
    class Container
    {
        /** @var T */
        public mixed $value;

        /** @param T $value */
        public function __construct(mixed $value)
        {
            $this->value = $value;
        }
    }

    /**
     * @param Container<Item> $c
     *
     * @return template-type<Container<Item>, Container::class, 'T'>
     */
    function extract_item(Container $c): object
    {
        return $c->value;
    }

    /**
     * @param Container<SubItem> $c
     *
     * @return template-type<Container<SubItem>, Container::class, 'T'>
     */
    function extract_subitem(Container $c): object
    {
        return $c->value;
    }

    function assertions(): void
    {
        $a = new Container(new Item());
        accept_item(extract_item($a));

        $b = new Container(new SubItem());
        accept_subitem(extract_subitem($b));
    }
}

namespace TemplateTypeMulti {
    class KeyType {}
    class ValueType {}

    function accept_key(KeyType $_): void {}
    function accept_value(ValueType $_): void {}

    /**
     * @template TKey of object
     * @template TValue of object
     */
    class Pair
    {
        /** @var TKey */
        public object $key;
        /** @var TValue */
        public object $value;

        /**
         * @param TKey $key
         * @param TValue $value
         */
        public function __construct(object $key, object $value)
        {
            $this->key = $key;
            $this->value = $value;
        }
    }

    /**
     * @param Pair<KeyType, ValueType> $pair
     *
     * @return template-type<Pair<KeyType, ValueType>, Pair::class, 'TKey'>
     */
    function extract_key(Pair $pair): object
    {
        return $pair->key;
    }

    /**
     * @param Pair<KeyType, ValueType> $pair
     *
     * @return template-type<Pair<KeyType, ValueType>, Pair::class, 'TValue'>
     */
    function extract_value(Pair $pair): object
    {
        return $pair->value;
    }

    function assertions(): void
    {
        $pair = new Pair(new KeyType(), new ValueType());
        accept_key(extract_key($pair));
        accept_value(extract_value($pair));
    }
}

namespace Discussion9053 {
    /**
     * @template TChild of ChildInterface
     */
    interface ModelInterface
    {
        /**
         * @return non-empty-list<TChild>
         */
        public function getChildren(): array;
    }

    /**
     * @implements ModelInterface<Child>
     */
    class Model implements ModelInterface
    {
        /**
         * @param non-empty-list<Child> $children
         */
        public function __construct(
            public array $children,
        ) {}

        #[\Override]
        public function getChildren(): array
        {
            return $this->children;
        }
    }

    /**
     * @template T of ModelInterface
     */
    interface ChildInterface
    {
        /**
         * @return T
         */
        public function getModel(): ModelInterface;
    }

    /**
     * @implements ChildInterface<Model>
     */
    class Child implements ChildInterface
    {
        public function __construct(private Model $model) {}

        #[\Override]
        public function getModel(): Model
        {
            return $this->model;
        }
    }

    /**
     * @template T of ModelInterface
     */
    class Helper
    {
        /**
         * @param T $model
         */
        public function __construct(private ModelInterface $model) {}

        /**
         * @return template-type<T, ModelInterface, 'TChild'>
         */
        public function getFirstChildren(): ChildInterface
        {
            return $this->model->getChildren()[0];
        }
    }

    function accept_child(Child $_): void {}

    function assertions(Child $c): void
    {
        $model = new Model([$c]);
        $helper = new Helper($model);
        accept_child($helper->getFirstChildren());
    }
}

namespace Bug13474 {
    /**
     * @template TValue of mixed
     */
    interface ModelInterface
    {
        /**
         * @return TValue
         */
        public function getValue(): mixed;
    }

    /**
     * @implements ModelInterface<int>
     */
    class ModelA implements ModelInterface
    {
        #[\Override]
        public function getValue(): int
        {
            return 0;
        }
    }

    /**
     * @implements ModelInterface<string>
     */
    class ModelB implements ModelInterface
    {
        #[\Override]
        public function getValue(): string
        {
            return 'foo';
        }
    }

    /**
     * @template T of ModelInterface
     */
    trait ModelTrait
    {
        /**
         * @return T
         */
        abstract public function model(): ModelInterface;

        /**
         * @return template-type<T, ModelInterface, 'TValue'>
         */
        public function getValue(): mixed
        {
            return $this->model()->getValue();
        }
    }

    class TestA
    {
        /** @use ModelTrait<ModelA> */
        use ModelTrait;

        #[\Override]
        public function model(): ModelA
        {
            return new ModelA();
        }
    }

    class TestB
    {
        /** @use ModelTrait<ModelB> */
        use ModelTrait;

        #[\Override]
        public function model(): ModelB
        {
            return new ModelB();
        }
    }

    function accept_int(int $_): void {}
    function accept_string(string $_): void {}

    function assertions(): void
    {
        $a = new TestA();
        accept_int($a->getValue());

        $b = new TestB();
        accept_string($b->getValue());
    }
}
