//! `EntityRepository::findBy()` / `findOneBy()` criteria-key validator.
//!
//! STATUS: WIP scaffold. Design:
//!
//! Target: methods `findBy`, `findOneBy`, `count` on any class that is an
//! instance of `Doctrine\ORM\EntityRepository` (wildcard class target, then
//! `context.is_instance_of(...)`, exactly like the psr_container provider).
//!
//! Resolution of the entity: `EntityRepository<T>` — `T` comes either from the
//! generic type of the receiver expression (already inferred by the analyzer
//! through doctrine/orm's own docblocks) or from the `::class` argument of the
//! `getRepository()` call one step up the chain.
//!
//! Field map: the codex has already parsed the entity class and its PHP
//! attributes. Persisted fields = properties carrying `#[ORM\Column]`,
//! `#[ORM\Id]`, `#[ORM\ManyToOne]`, `#[ORM\OneToOne]`, `#[ORM\OneToMany]`,
//! `#[ORM\ManyToMany]`, `#[ORM\Embedded]`. XML/YAML mappings are out of scope
//! for the first iteration (attribute mapping is the modern default).
//!
//! Check: for a literal array criteria argument, every string key must be in
//! the field map. Unknown key -> report `doctrine-unknown-field` with the
//! entity name and the closest match (levenshtein over the field map).
//!
//! Acceptance fixture (from a production codebase — see PLUGINS-RFC.md):
//!   $em->getRepository(UserEntity::class)->findOneBy(['champInexistant' => 'x']);
//! must report `doctrine-unknown-field`, matching what phpstan-doctrine
//! reports as `doctrine.findOneByArgument`.

/// Validator for criteria keys passed to Doctrine repository finder methods.
#[derive(Default)]
pub struct FindByFieldValidator;
