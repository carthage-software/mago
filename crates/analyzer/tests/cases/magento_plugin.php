<?php

declare(strict_types=1);

// === Magento stubs ===

namespace Magento\Framework\Model {
    abstract class AbstractModel
    {
        public function save(): self { return $this; }
        public function load(int $id): self { return $this; }
        public function delete(): self { return $this; }
        public function getCollection(): object { return new \stdClass(); }
        public function getResource(): object { return new \stdClass(); }
        public function _getResource(): object { return new \stdClass(); }
    }
}

namespace Magento\Framework\View\Element {
    class Template
    {
        public function setTemplate(string $template): self { return $this; }
    }
}

namespace Magento\Framework {
    interface ObjectManagerInterface
    {
        /**
         * @template T of object
         * @param class-string<T> $type
         * @return T
         */
        public function get(string $type): object;

        /**
         * @template T of object
         * @param class-string<T> $type
         * @return T
         */
        public function create(string $type): object;
    }
}

namespace Magento\Framework\Data {
    class Collection {}
}

namespace Magento\Framework\TestFramework\Unit\Helper {
    class ObjectManager
    {
        public function getCollectionMock(string $className, array $data = []): object
        {
            return new \stdClass();
        }
    }
}

// === Test models ===

namespace Vendor\Module\Model {
    use Magento\Framework\Model\AbstractModel;

    class Product extends AbstractModel {}
}

namespace Vendor\Module\Block {
    use Magento\Framework\View\Element\Template;

    class ProductView extends Template {}
}

namespace Vendor\Module\Model\ResourceModel\Product {
    use Magento\Framework\Data\Collection;

    class CollectionValid extends Collection {}
}

namespace Vendor\Module\Model\ResourceModel\Product {
    class CollectionInvalid {}
}

// === Tests for method call hooks ===

namespace Vendor\Module\Test {
    use Vendor\Module\Model\Product;
    use Vendor\Module\Block\ProductView;

    function test_use_service_contracts(Product $product): void
    {
        /** @mago-expect analysis:magento-use-service-contracts */
        $product->save();

        /** @mago-expect analysis:magento-use-service-contracts */
        $product->load(1);

        /** @mago-expect analysis:magento-use-service-contracts */
        $product->delete();
    }

    function test_collection_via_factory(Product $product): void
    {
        /** @mago-expect analysis:magento-collection-via-factory */
        $product->getCollection();
    }

    function test_use_resource_model_directly(Product $product): void
    {
        /** @mago-expect analysis:magento-use-resource-model-directly */
        $product->getResource();

        /** @mago-expect analysis:magento-use-resource-model-directly */
        $product->_getResource();
    }

    function test_no_set_template_in_block(ProductView $block): void
    {
        /** @mago-expect analysis:magento-no-set-template-in-block */
        $block->setTemplate('Vendor_Module::product/view.phtml');
    }

    function test_object_manager_return_type(\Magento\Framework\ObjectManagerInterface $om): void
    {
        // ObjectManager::get should return the correct type (Product)
        $product = $om->get(Product::class);

        // Since $product is resolved as Product (extends AbstractModel), save() is flagged
        /** @mago-expect analysis:magento-use-service-contracts */
        $product->save();
    }
}
