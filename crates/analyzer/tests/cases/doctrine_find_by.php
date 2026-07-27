<?php

declare(strict_types=1);

namespace Doctrine\ORM\Mapping {
    #[\Attribute(\Attribute::TARGET_CLASS)]
    final class Entity
    {
    }

    #[\Attribute(\Attribute::TARGET_PROPERTY)]
    final class Column
    {
        public function __construct(
            public ?string $type = null,
        ) {}
    }

    #[\Attribute(\Attribute::TARGET_PROPERTY)]
    final class Id
    {
    }

    #[\Attribute(\Attribute::TARGET_PROPERTY)]
    final class ManyToOne
    {
    }
}

namespace Doctrine\ORM {
    /**
     * Doctrine ORM EntityRepository stub for testing.
     *
     * @template T of object
     */
    class EntityRepository
    {
        /**
         * @return list<T>
         */
        public function findBy(array $criteria, ?array $orderBy = null, ?int $limit = null, ?int $offset = null): array
        {
            return [];
        }

        /**
         * @return T|null
         */
        public function findOneBy(array $criteria, ?array $orderBy = null): ?object
        {
            return null;
        }

        public function count(array $criteria = []): int
        {
            return 0;
        }
    }

    interface EntityManagerInterface
    {
        /**
         * @template TEntity of object
         * @param class-string<TEntity> $className
         * @return EntityRepository<TEntity>
         */
        public function getRepository(string $className): EntityRepository;
    }
}

namespace App {
    use Doctrine\ORM\EntityManagerInterface;
    use Doctrine\ORM\EntityRepository;
    use Doctrine\ORM\Mapping as ORM;

    #[ORM\Entity]
    class UserEntity
    {
        #[ORM\Id]
        #[ORM\Column(type: 'uuid')]
        public string $id = '';

        #[ORM\Column]
        public string $email = '';

        #[ORM\Column]
        public bool $active = false;

        #[ORM\ManyToOne]
        public ?UserEntity $manager = null;

        public string $notMapped = '';
    }

    /**
     * @extends EntityRepository<UserEntity>
     */
    class UserRepository extends EntityRepository
    {
    }

    class PlainEntity
    {
        public string $anything = '';
    }

    function unknown_field_is_reported(EntityManagerInterface $em): void
    {
        /** @mago-expect analysis:doctrine-unknown-field */
        $em->getRepository(UserEntity::class)->findOneBy(['champInexistant' => 'x']);

        /** @mago-expect analysis:doctrine-unknown-field */
        $em->getRepository(UserEntity::class)->findBy(['emaill' => 'typo-gets-a-suggestion']);

        /** @mago-expect analysis:doctrine-unknown-field */
        $em->getRepository(UserEntity::class)->count(['nope' => 1]);
    }

    function unknown_field_through_custom_repository(UserRepository $repository): void
    {
        /** @mago-expect analysis:doctrine-unknown-field */
        $repository->findOneBy(['missing' => 'x']);
    }

    function valid_fields_are_silent(EntityManagerInterface $em, UserRepository $repository): void
    {
        $em->getRepository(UserEntity::class)->findOneBy(['email' => 'jane@example.com']);
        $em->getRepository(UserEntity::class)->findBy(['active' => true], ['email' => 'ASC']);
        $em->getRepository(UserEntity::class)->count(['id' => 'some-uuid']);
        $repository->findOneBy(['manager' => null]);
    }

    /**
     * @param array<string, mixed> $criteria
     */
    function non_literal_criteria_is_no_opinion(UserRepository $repository, array $criteria): void
    {
        $repository->findBy($criteria);
    }

    function unsupported_keys_are_no_opinion(UserRepository $repository): void
    {
        // Nested paths are only handled by some repository extensions.
        $repository->findBy(['manager.email' => 'x']);

        // Non-string keys are left to regular argument analysis.
        $repository->findBy([0 => 'x']);
    }

    /**
     * @param EntityRepository<PlainEntity> $repository
     */
    function entity_without_attribute_mapping_is_no_opinion(EntityRepository $repository): void
    {
        // PlainEntity has no ORM attributes (e.g. XML/YAML mapping): no opinion.
        $repository->findBy(['whatever' => 'x']);
    }
}
