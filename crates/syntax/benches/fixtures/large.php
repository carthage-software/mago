<?php

declare(strict_types=1);

namespace Benchmark\Large;

use Attribute;
use ArrayAccess;
use ArrayIterator;
use Closure;
use Countable;
use Exception;
use Generator;
use InvalidArgumentException;
use IteratorAggregate;
use JsonSerializable;
use LogicException;
use OutOfBoundsException;
use OverflowException;
use RuntimeException;
use Stringable;
use Throwable;
use Traversable;
use UnderflowException;
use WeakMap;
use WeakReference;

// ============================================================================
// ATTRIBUTES
// ============================================================================

#[Attribute(Attribute::TARGET_CLASS)]
final class Entity
{
    public function __construct(
        public readonly string $table,
        public readonly ?string $schema = null,
        public readonly array $indexes = [],
    ) {}
}

#[Attribute(Attribute::TARGET_PROPERTY)]
final class Column
{
    public function __construct(
        public readonly string $name,
        public readonly string $type = 'string',
        public readonly bool $nullable = false,
        public readonly bool $unique = false,
        public readonly ?string $default = null,
        public readonly ?int $length = null,
    ) {}
}

#[Attribute(Attribute::TARGET_PROPERTY)]
final class PrimaryKey
{
    public function __construct(
        public readonly bool $autoIncrement = true,
    ) {}
}

#[Attribute(Attribute::TARGET_PROPERTY)]
final class ForeignKey
{
    public function __construct(
        public readonly string $references,
        public readonly string $on,
        public readonly string $onDelete = 'CASCADE',
        public readonly string $onUpdate = 'CASCADE',
    ) {}
}

#[Attribute(Attribute::TARGET_METHOD)]
final class Route
{
    public function __construct(
        public readonly string $path,
        public readonly string $method = 'GET',
        public readonly array $middleware = [],
        public readonly ?string $name = null,
    ) {}
}

#[Attribute(Attribute::TARGET_METHOD | Attribute::TARGET_CLASS)]
final class Middleware
{
    public function __construct(
        public readonly string|array $middleware,
        public readonly int $priority = 0,
    ) {}
}

#[Attribute(Attribute::TARGET_PARAMETER)]
final class Validate
{
    public function __construct(
        public readonly array $rules,
        public readonly ?string $message = null,
    ) {}
}

#[Attribute(Attribute::TARGET_METHOD)]
final class Cache
{
    public function __construct(
        public readonly int $ttl = 3600,
        public readonly ?string $key = null,
        public readonly array $tags = [],
    ) {}
}

#[Attribute(Attribute::TARGET_METHOD)]
final class Transactional
{
    public function __construct(
        public readonly string $isolation = 'READ_COMMITTED',
        public readonly bool $readOnly = false,
    ) {}
}

#[Attribute(Attribute::TARGET_CLASS | Attribute::TARGET_METHOD)]
final class Deprecated
{
    public function __construct(
        public readonly string $message,
        public readonly ?string $since = null,
        public readonly ?string $replacement = null,
    ) {}
}

// ============================================================================
// INTERFACES
// ============================================================================

interface RepositoryInterface
{
    public function find(int $id): ?object;
    public function findBy(array $criteria, ?array $orderBy = null, ?int $limit = null, ?int $offset = null): array;
    public function findOneBy(array $criteria): ?object;
    public function findAll(): array;
    public function count(array $criteria = []): int;
    public function save(object $entity): void;
    public function delete(object $entity): void;
    public function flush(): void;
}

interface CacheInterface
{
    public function get(string $key, mixed $default = null): mixed;
    public function set(string $key, mixed $value, ?int $ttl = null): bool;
    public function has(string $key): bool;
    public function delete(string $key): bool;
    public function clear(): bool;
    public function getMultiple(iterable $keys, mixed $default = null): iterable;
    public function setMultiple(iterable $values, ?int $ttl = null): bool;
    public function deleteMultiple(iterable $keys): bool;
}

interface LoggerInterface
{
    public function emergency(string|\Stringable $message, array $context = []): void;
    public function alert(string|\Stringable $message, array $context = []): void;
    public function critical(string|\Stringable $message, array $context = []): void;
    public function error(string|\Stringable $message, array $context = []): void;
    public function warning(string|\Stringable $message, array $context = []): void;
    public function notice(string|\Stringable $message, array $context = []): void;
    public function info(string|\Stringable $message, array $context = []): void;
    public function debug(string|\Stringable $message, array $context = []): void;
    public function log(mixed $level, string|\Stringable $message, array $context = []): void;
}

interface EventDispatcherInterface
{
    public function dispatch(object $event): object;
    public function addListener(string $eventName, callable $listener, int $priority = 0): void;
    public function removeListener(string $eventName, callable $listener): void;
    public function getListeners(?string $eventName = null): array;
    public function hasListeners(?string $eventName = null): bool;
}

interface MessageBusInterface
{
    public function dispatch(object $message, array $stamps = []): Envelope;
}

interface SerializerInterface
{
    public function serialize(mixed $data, string $format, array $context = []): string;
    public function deserialize(string $data, string $type, string $format, array $context = []): mixed;
}

interface ValidatorInterface
{
    public function validate(mixed $value, array $constraints = [], array $groups = []): ConstraintViolationListInterface;
    public function validateProperty(object $object, string $propertyName, array $groups = []): ConstraintViolationListInterface;
}

interface HttpClientInterface
{
    public function request(string $method, string $url, array $options = []): ResponseInterface;
    public function get(string $url, array $options = []): ResponseInterface;
    public function post(string $url, array $options = []): ResponseInterface;
    public function put(string $url, array $options = []): ResponseInterface;
    public function patch(string $url, array $options = []): ResponseInterface;
    public function delete(string $url, array $options = []): ResponseInterface;
}

interface ContainerInterface
{
    public function get(string $id): mixed;
    public function has(string $id): bool;
    public function set(string $id, mixed $value): void;
    public function make(string $abstract, array $parameters = []): mixed;
    public function bind(string $abstract, Closure|string|null $concrete = null, bool $shared = false): void;
    public function singleton(string $abstract, Closure|string|null $concrete = null): void;
    public function instance(string $abstract, mixed $instance): mixed;
}

// ============================================================================
// TRAITS
// ============================================================================

trait TimestampTrait
{
    #[Column(name: 'created_at', type: 'datetime')]
    private ?\DateTimeImmutable $createdAt = null;

    #[Column(name: 'updated_at', type: 'datetime', nullable: true)]
    private ?\DateTimeImmutable $updatedAt = null;

    public function getCreatedAt(): ?\DateTimeImmutable
    {
        return $this->createdAt;
    }

    public function setCreatedAt(\DateTimeImmutable $createdAt): static
    {
        $this->createdAt = $createdAt;
        return $this;
    }

    public function getUpdatedAt(): ?\DateTimeImmutable
    {
        return $this->updatedAt;
    }

    public function setUpdatedAt(?\DateTimeImmutable $updatedAt): static
    {
        $this->updatedAt = $updatedAt;
        return $this;
    }

    public function touch(): static
    {
        $this->updatedAt = new \DateTimeImmutable();
        return $this;
    }
}

trait SoftDeleteTrait
{
    #[Column(name: 'deleted_at', type: 'datetime', nullable: true)]
    private ?\DateTimeImmutable $deletedAt = null;

    public function getDeletedAt(): ?\DateTimeImmutable
    {
        return $this->deletedAt;
    }

    public function isDeleted(): bool
    {
        return $this->deletedAt !== null;
    }

    public function delete(): static
    {
        $this->deletedAt = new \DateTimeImmutable();
        return $this;
    }

    public function restore(): static
    {
        $this->deletedAt = null;
        return $this;
    }
}

trait UuidTrait
{
    #[Column(name: 'uuid', type: 'uuid', unique: true)]
    private ?string $uuid = null;

    public function getUuid(): ?string
    {
        return $this->uuid;
    }

    public function generateUuid(): static
    {
        $this->uuid = sprintf(
            '%04x%04x-%04x-%04x-%04x-%04x%04x%04x',
            mt_rand(0, 0xffff),
            mt_rand(0, 0xffff),
            mt_rand(0, 0xffff),
            mt_rand(0, 0x0fff) | 0x4000,
            mt_rand(0, 0x3fff) | 0x8000,
            mt_rand(0, 0xffff),
            mt_rand(0, 0xffff),
            mt_rand(0, 0xffff)
        );
        return $this;
    }
}

trait JsonSerializableTrait
{
    public function jsonSerialize(): array
    {
        return $this->toArray();
    }

    abstract public function toArray(): array;
}

trait SingletonTrait
{
    private static ?self $instance = null;

    private function __construct() {}

    private function __clone(): void {}

    public function __wakeup(): void
    {
        throw new LogicException('Cannot unserialize singleton');
    }

    public static function getInstance(): static
    {
        return static::$instance ??= new static();
    }

    public static function resetInstance(): void
    {
        static::$instance = null;
    }
}

trait EventEmitterTrait
{
    private array $listeners = [];

    public function on(string $event, callable $listener, int $priority = 0): static
    {
        $this->listeners[$event][$priority][] = $listener;
        return $this;
    }

    public function off(string $event, ?callable $listener = null): static
    {
        if ($listener === null) {
            unset($this->listeners[$event]);
        } else {
            foreach ($this->listeners[$event] ?? [] as $priority => $listeners) {
                $key = array_search($listener, $listeners, true);
                if ($key !== false) {
                    unset($this->listeners[$event][$priority][$key]);
                }
            }
        }
        return $this;
    }

    public function emit(string $event, mixed ...$args): static
    {
        if (!isset($this->listeners[$event])) {
            return $this;
        }

        krsort($this->listeners[$event]);
        foreach ($this->listeners[$event] as $listeners) {
            foreach ($listeners as $listener) {
                $result = $listener(...$args);
                if ($result === false) {
                    break 2;
                }
            }
        }
        return $this;
    }
}

// ============================================================================
// ENUMS
// ============================================================================

enum Status: string
{
    case Draft = 'draft';
    case Pending = 'pending';
    case Active = 'active';
    case Inactive = 'inactive';
    case Archived = 'archived';
    case Deleted = 'deleted';

    public function label(): string
    {
        return match ($this) {
            self::Draft => 'Draft',
            self::Pending => 'Pending Review',
            self::Active => 'Active',
            self::Inactive => 'Inactive',
            self::Archived => 'Archived',
            self::Deleted => 'Deleted',
        };
    }

    public function color(): string
    {
        return match ($this) {
            self::Draft => 'gray',
            self::Pending => 'yellow',
            self::Active => 'green',
            self::Inactive => 'orange',
            self::Archived => 'blue',
            self::Deleted => 'red',
        };
    }

    public function isEditable(): bool
    {
        return match ($this) {
            self::Draft, self::Pending, self::Active, self::Inactive => true,
            self::Archived, self::Deleted => false,
        };
    }

    public function isVisible(): bool
    {
        return match ($this) {
            self::Active => true,
            default => false,
        };
    }

    public static function editable(): array
    {
        return array_filter(self::cases(), fn(self $status) => $status->isEditable());
    }

    public static function fromString(string $value): self
    {
        return match (strtolower($value)) {
            'draft' => self::Draft,
            'pending' => self::Pending,
            'active' => self::Active,
            'inactive' => self::Inactive,
            'archived' => self::Archived,
            'deleted' => self::Deleted,
            default => throw new InvalidArgumentException("Unknown status: {$value}"),
        };
    }
}

enum Priority: int
{
    case Lowest = 0;
    case Low = 1;
    case Medium = 2;
    case High = 3;
    case Highest = 4;
    case Critical = 5;

    public function label(): string
    {
        return match ($this) {
            self::Lowest => 'Lowest',
            self::Low => 'Low',
            self::Medium => 'Medium',
            self::High => 'High',
            self::Highest => 'Highest',
            self::Critical => 'Critical',
        };
    }

    public function icon(): string
    {
        return match ($this) {
            self::Lowest, self::Low => 'arrow-down',
            self::Medium => 'minus',
            self::High, self::Highest => 'arrow-up',
            self::Critical => 'exclamation',
        };
    }

    public static function fromValue(int $value): self
    {
        return match ($value) {
            0 => self::Lowest,
            1 => self::Low,
            2 => self::Medium,
            3 => self::High,
            4 => self::Highest,
            5 => self::Critical,
            default => throw new InvalidArgumentException("Invalid priority value: {$value}"),
        };
    }
}

enum HttpMethod: string
{
    case GET = 'GET';
    case POST = 'POST';
    case PUT = 'PUT';
    case PATCH = 'PATCH';
    case DELETE = 'DELETE';
    case HEAD = 'HEAD';
    case OPTIONS = 'OPTIONS';
    case TRACE = 'TRACE';
    case CONNECT = 'CONNECT';

    public function isSafe(): bool
    {
        return match ($this) {
            self::GET, self::HEAD, self::OPTIONS, self::TRACE => true,
            default => false,
        };
    }

    public function isIdempotent(): bool
    {
        return match ($this) {
            self::GET, self::HEAD, self::OPTIONS, self::TRACE, self::PUT, self::DELETE => true,
            default => false,
        };
    }

    public function allowsBody(): bool
    {
        return match ($this) {
            self::POST, self::PUT, self::PATCH => true,
            default => false,
        };
    }
}

enum LogLevel: int
{
    case Emergency = 0;
    case Alert = 1;
    case Critical = 2;
    case Error = 3;
    case Warning = 4;
    case Notice = 5;
    case Info = 6;
    case Debug = 7;

    public function name(): string
    {
        return match ($this) {
            self::Emergency => 'EMERGENCY',
            self::Alert => 'ALERT',
            self::Critical => 'CRITICAL',
            self::Error => 'ERROR',
            self::Warning => 'WARNING',
            self::Notice => 'NOTICE',
            self::Info => 'INFO',
            self::Debug => 'DEBUG',
        };
    }

    public function includes(self $level): bool
    {
        return $level->value <= $this->value;
    }
}

enum Permission: string
{
    case Read = 'read';
    case Write = 'write';
    case Delete = 'delete';
    case Admin = 'admin';
    case SuperAdmin = 'super_admin';

    public function implies(): array
    {
        return match ($this) {
            self::Read => [],
            self::Write => [self::Read],
            self::Delete => [self::Read, self::Write],
            self::Admin => [self::Read, self::Write, self::Delete],
            self::SuperAdmin => [self::Read, self::Write, self::Delete, self::Admin],
        };
    }

    public function hasPermission(self $permission): bool
    {
        return $this === $permission || in_array($permission, $this->implies(), true);
    }
}

// ============================================================================
// ABSTRACT CLASSES
// ============================================================================

abstract class AbstractEntity implements Stringable, JsonSerializable
{
    use TimestampTrait;
    use JsonSerializableTrait;

    #[PrimaryKey]
    #[Column(name: 'id', type: 'integer')]
    protected ?int $id = null;

    public function getId(): ?int
    {
        return $this->id;
    }

    public function isPersisted(): bool
    {
        return $this->id !== null;
    }

    abstract public function validate(): bool;

    abstract public function toArray(): array;

    public function __toString(): string
    {
        return sprintf('%s#%s', static::class, $this->id ?? 'new');
    }

    public function equals(?self $other): bool
    {
        if ($other === null) {
            return false;
        }

        if ($this === $other) {
            return true;
        }

        if (get_class($this) !== get_class($other)) {
            return false;
        }

        return $this->id !== null && $this->id === $other->id;
    }
}

abstract class AbstractRepository implements RepositoryInterface
{
    protected string $entityClass;
    protected array $entities = [];

    public function __construct(
        protected readonly ConnectionInterface $connection,
        protected readonly LoggerInterface $logger,
    ) {}

    public function find(int $id): ?object
    {
        return $this->entities[$id] ?? null;
    }

    public function findBy(
        array $criteria,
        ?array $orderBy = null,
        ?int $limit = null,
        ?int $offset = null,
    ): array {
        $result = array_filter(
            $this->entities,
            fn($entity) => $this->matchesCriteria($entity, $criteria)
        );

        if ($orderBy !== null) {
            $result = $this->applyOrderBy($result, $orderBy);
        }

        if ($offset !== null) {
            $result = array_slice($result, $offset);
        }

        if ($limit !== null) {
            $result = array_slice($result, 0, $limit);
        }

        return array_values($result);
    }

    public function findOneBy(array $criteria): ?object
    {
        $results = $this->findBy($criteria, limit: 1);
        return $results[0] ?? null;
    }

    public function findAll(): array
    {
        return array_values($this->entities);
    }

    public function count(array $criteria = []): int
    {
        return count($this->findBy($criteria));
    }

    public function save(object $entity): void
    {
        if (!$entity instanceof AbstractEntity) {
            throw new InvalidArgumentException('Entity must extend AbstractEntity');
        }

        if ($entity->getId() === null) {
            $id = count($this->entities) + 1;
            $entity->setCreatedAt(new \DateTimeImmutable());
            $this->entities[$id] = $entity;
        } else {
            $entity->touch();
        }

        $this->logger->debug('Entity saved', [
            'class' => get_class($entity),
            'id' => $entity->getId(),
        ]);
    }

    public function delete(object $entity): void
    {
        if (!$entity instanceof AbstractEntity) {
            throw new InvalidArgumentException('Entity must extend AbstractEntity');
        }

        if ($entity->getId() !== null) {
            unset($this->entities[$entity->getId()]);
        }
    }

    public function flush(): void
    {
        // Persist changes to database
        $this->logger->info('Repository flushed', [
            'entity_count' => count($this->entities),
        ]);
    }

    protected function matchesCriteria(object $entity, array $criteria): bool
    {
        foreach ($criteria as $key => $value) {
            $getter = 'get' . ucfirst($key);
            if (!method_exists($entity, $getter)) {
                return false;
            }
            if ($entity->$getter() !== $value) {
                return false;
            }
        }
        return true;
    }

    protected function applyOrderBy(array $entities, array $orderBy): array
    {
        usort($entities, function ($a, $b) use ($orderBy) {
            foreach ($orderBy as $field => $direction) {
                $getter = 'get' . ucfirst($field);
                $valueA = $a->$getter();
                $valueB = $b->$getter();

                $cmp = $valueA <=> $valueB;
                if ($cmp !== 0) {
                    return strtoupper($direction) === 'DESC' ? -$cmp : $cmp;
                }
            }
            return 0;
        });

        return $entities;
    }
}

abstract class AbstractController
{
    public function __construct(
        protected readonly ContainerInterface $container,
        protected readonly LoggerInterface $logger,
    ) {}

    protected function json(mixed $data, int $status = 200, array $headers = []): Response
    {
        return new JsonResponse($data, $status, $headers);
    }

    protected function render(string $template, array $context = [], int $status = 200): Response
    {
        $content = $this->container->get('twig')->render($template, $context);
        return new Response($content, $status, ['Content-Type' => 'text/html']);
    }

    protected function redirect(string $url, int $status = 302): Response
    {
        return new RedirectResponse($url, $status);
    }

    protected function notFound(string $message = 'Not Found'): never
    {
        throw new NotFoundException($message);
    }

    protected function forbidden(string $message = 'Forbidden'): never
    {
        throw new ForbiddenException($message);
    }

    protected function badRequest(string $message = 'Bad Request'): never
    {
        throw new BadRequestException($message);
    }
}

abstract class AbstractCommand
{
    protected InputInterface $input;
    protected OutputInterface $output;

    abstract protected function configure(): void;
    abstract protected function execute(): int;

    public function run(InputInterface $input, OutputInterface $output): int
    {
        $this->input = $input;
        $this->output = $output;

        try {
            return $this->execute();
        } catch (Throwable $e) {
            $this->output->error($e->getMessage());
            return 1;
        }
    }

    protected function argument(string $name): mixed
    {
        return $this->input->getArgument($name);
    }

    protected function option(string $name): mixed
    {
        return $this->input->getOption($name);
    }

    protected function info(string $message): void
    {
        $this->output->writeln("<info>{$message}</info>");
    }

    protected function error(string $message): void
    {
        $this->output->writeln("<error>{$message}</error>");
    }

    protected function warning(string $message): void
    {
        $this->output->writeln("<comment>{$message}</comment>");
    }

    protected function success(string $message): void
    {
        $this->output->writeln("<info>✓ {$message}</info>");
    }

    protected function confirm(string $question, bool $default = false): bool
    {
        return $this->output->confirm($question, $default);
    }

    protected function ask(string $question, ?string $default = null): ?string
    {
        return $this->output->ask($question, $default);
    }

    protected function table(array $headers, array $rows): void
    {
        $this->output->table($headers, $rows);
    }

    protected function progressStart(int $max): void
    {
        $this->output->progressStart($max);
    }

    protected function progressAdvance(int $step = 1): void
    {
        $this->output->progressAdvance($step);
    }

    protected function progressFinish(): void
    {
        $this->output->progressFinish();
    }
}

abstract class AbstractValueObject implements Stringable
{
    abstract public function equals(self $other): bool;
    abstract public function __toString(): string;

    public function __serialize(): array
    {
        return get_object_vars($this);
    }

    public function __unserialize(array $data): void
    {
        foreach ($data as $key => $value) {
            $this->$key = $value;
        }
    }
}

abstract class AbstractEvent
{
    private bool $propagationStopped = false;

    public function stopPropagation(): void
    {
        $this->propagationStopped = true;
    }

    public function isPropagationStopped(): bool
    {
        return $this->propagationStopped;
    }
}

// ============================================================================
// CONCRETE CLASSES
// ============================================================================

#[Entity(table: 'users')]
class User extends AbstractEntity
{
    use SoftDeleteTrait;
    use UuidTrait;

    #[Column(name: 'email', type: 'string', unique: true, length: 255)]
    private string $email;

    #[Column(name: 'password_hash', type: 'string', length: 255)]
    private string $passwordHash;

    #[Column(name: 'name', type: 'string', length: 100)]
    private string $name;

    #[Column(name: 'status', type: 'string', length: 20)]
    private Status $status;

    #[Column(name: 'roles', type: 'json')]
    private array $roles = [];

    #[Column(name: 'metadata', type: 'json', nullable: true)]
    private ?array $metadata = null;

    #[Column(name: 'last_login_at', type: 'datetime', nullable: true)]
    private ?\DateTimeImmutable $lastLoginAt = null;

    public function __construct(
        string $email,
        string $name,
        string $password,
        Status $status = Status::Pending,
    ) {
        $this->email = $email;
        $this->name = $name;
        $this->passwordHash = password_hash($password, PASSWORD_ARGON2ID);
        $this->status = $status;
        $this->createdAt = new \DateTimeImmutable();
        $this->generateUuid();
    }

    public function getEmail(): string
    {
        return $this->email;
    }

    public function setEmail(string $email): static
    {
        if (!filter_var($email, FILTER_VALIDATE_EMAIL)) {
            throw new InvalidArgumentException("Invalid email address: {$email}");
        }
        $this->email = $email;
        return $this->touch();
    }

    public function getName(): string
    {
        return $this->name;
    }

    public function setName(string $name): static
    {
        if (strlen($name) < 2 || strlen($name) > 100) {
            throw new InvalidArgumentException('Name must be between 2 and 100 characters');
        }
        $this->name = $name;
        return $this->touch();
    }

    public function getStatus(): Status
    {
        return $this->status;
    }

    public function setStatus(Status $status): static
    {
        $this->status = $status;
        return $this->touch();
    }

    public function getRoles(): array
    {
        return $this->roles;
    }

    public function addRole(string $role): static
    {
        if (!in_array($role, $this->roles, true)) {
            $this->roles[] = $role;
        }
        return $this;
    }

    public function removeRole(string $role): static
    {
        $this->roles = array_values(array_filter(
            $this->roles,
            fn($r) => $r !== $role
        ));
        return $this;
    }

    public function hasRole(string $role): bool
    {
        return in_array($role, $this->roles, true);
    }

    public function getMetadata(): ?array
    {
        return $this->metadata;
    }

    public function setMetadata(?array $metadata): static
    {
        $this->metadata = $metadata;
        return $this->touch();
    }

    public function getMetadataValue(string $key, mixed $default = null): mixed
    {
        return $this->metadata[$key] ?? $default;
    }

    public function setMetadataValue(string $key, mixed $value): static
    {
        $this->metadata ??= [];
        $this->metadata[$key] = $value;
        return $this->touch();
    }

    public function getLastLoginAt(): ?\DateTimeImmutable
    {
        return $this->lastLoginAt;
    }

    public function recordLogin(): static
    {
        $this->lastLoginAt = new \DateTimeImmutable();
        return $this;
    }

    public function verifyPassword(string $password): bool
    {
        return password_verify($password, $this->passwordHash);
    }

    public function setPassword(string $password): static
    {
        if (strlen($password) < 8) {
            throw new InvalidArgumentException('Password must be at least 8 characters');
        }
        $this->passwordHash = password_hash($password, PASSWORD_ARGON2ID);
        return $this->touch();
    }

    public function validate(): bool
    {
        return !empty($this->email)
            && filter_var($this->email, FILTER_VALIDATE_EMAIL) !== false
            && !empty($this->name)
            && strlen($this->name) >= 2
            && !empty($this->passwordHash);
    }

    public function toArray(): array
    {
        return [
            'id' => $this->id,
            'uuid' => $this->uuid,
            'email' => $this->email,
            'name' => $this->name,
            'status' => $this->status->value,
            'roles' => $this->roles,
            'metadata' => $this->metadata,
            'last_login_at' => $this->lastLoginAt?->format('c'),
            'created_at' => $this->createdAt?->format('c'),
            'updated_at' => $this->updatedAt?->format('c'),
            'deleted_at' => $this->deletedAt?->format('c'),
        ];
    }
}

#[Entity(table: 'posts')]
class Post extends AbstractEntity
{
    use SoftDeleteTrait;
    use UuidTrait;

    #[Column(name: 'title', type: 'string', length: 255)]
    private string $title;

    #[Column(name: 'slug', type: 'string', unique: true, length: 255)]
    private string $slug;

    #[Column(name: 'content', type: 'text')]
    private string $content;

    #[Column(name: 'excerpt', type: 'text', nullable: true)]
    private ?string $excerpt = null;

    #[ForeignKey(references: 'id', on: 'users')]
    #[Column(name: 'author_id', type: 'integer')]
    private int $authorId;

    #[Column(name: 'status', type: 'string', length: 20)]
    private Status $status;

    #[Column(name: 'priority', type: 'integer')]
    private Priority $priority;

    #[Column(name: 'published_at', type: 'datetime', nullable: true)]
    private ?\DateTimeImmutable $publishedAt = null;

    #[Column(name: 'tags', type: 'json')]
    private array $tags = [];

    #[Column(name: 'metadata', type: 'json', nullable: true)]
    private ?array $metadata = null;

    #[Column(name: 'view_count', type: 'integer')]
    private int $viewCount = 0;

    public function __construct(
        string $title,
        string $content,
        int $authorId,
        Status $status = Status::Draft,
        Priority $priority = Priority::Medium,
    ) {
        $this->title = $title;
        $this->slug = $this->generateSlug($title);
        $this->content = $content;
        $this->authorId = $authorId;
        $this->status = $status;
        $this->priority = $priority;
        $this->createdAt = new \DateTimeImmutable();
        $this->generateUuid();
    }

    private function generateSlug(string $title): string
    {
        $slug = strtolower($title);
        $slug = preg_replace('/[^a-z0-9]+/', '-', $slug);
        $slug = trim($slug, '-');
        return $slug . '-' . substr(md5(uniqid()), 0, 8);
    }

    public function getTitle(): string
    {
        return $this->title;
    }

    public function setTitle(string $title): static
    {
        $this->title = $title;
        return $this->touch();
    }

    public function getSlug(): string
    {
        return $this->slug;
    }

    public function getContent(): string
    {
        return $this->content;
    }

    public function setContent(string $content): static
    {
        $this->content = $content;
        return $this->touch();
    }

    public function getExcerpt(): ?string
    {
        return $this->excerpt;
    }

    public function setExcerpt(?string $excerpt): static
    {
        $this->excerpt = $excerpt;
        return $this->touch();
    }

    public function getGeneratedExcerpt(int $length = 200): string
    {
        if ($this->excerpt !== null) {
            return $this->excerpt;
        }

        $text = strip_tags($this->content);
        if (strlen($text) <= $length) {
            return $text;
        }

        return substr($text, 0, $length) . '...';
    }

    public function getAuthorId(): int
    {
        return $this->authorId;
    }

    public function getStatus(): Status
    {
        return $this->status;
    }

    public function setStatus(Status $status): static
    {
        $this->status = $status;

        if ($status === Status::Active && $this->publishedAt === null) {
            $this->publishedAt = new \DateTimeImmutable();
        }

        return $this->touch();
    }

    public function getPriority(): Priority
    {
        return $this->priority;
    }

    public function setPriority(Priority $priority): static
    {
        $this->priority = $priority;
        return $this->touch();
    }

    public function getPublishedAt(): ?\DateTimeImmutable
    {
        return $this->publishedAt;
    }

    public function publish(): static
    {
        $this->status = Status::Active;
        $this->publishedAt = new \DateTimeImmutable();
        return $this->touch();
    }

    public function unpublish(): static
    {
        $this->status = Status::Draft;
        return $this->touch();
    }

    public function isPublished(): bool
    {
        return $this->status === Status::Active && $this->publishedAt !== null;
    }

    public function getTags(): array
    {
        return $this->tags;
    }

    public function setTags(array $tags): static
    {
        $this->tags = array_values(array_unique(array_filter($tags)));
        return $this->touch();
    }

    public function addTag(string $tag): static
    {
        if (!in_array($tag, $this->tags, true)) {
            $this->tags[] = $tag;
        }
        return $this;
    }

    public function removeTag(string $tag): static
    {
        $this->tags = array_values(array_filter(
            $this->tags,
            fn($t) => $t !== $tag
        ));
        return $this;
    }

    public function hasTag(string $tag): bool
    {
        return in_array($tag, $this->tags, true);
    }

    public function getMetadata(): ?array
    {
        return $this->metadata;
    }

    public function setMetadata(?array $metadata): static
    {
        $this->metadata = $metadata;
        return $this->touch();
    }

    public function getViewCount(): int
    {
        return $this->viewCount;
    }

    public function incrementViewCount(): static
    {
        $this->viewCount++;
        return $this;
    }

    public function validate(): bool
    {
        return !empty($this->title)
            && !empty($this->content)
            && $this->authorId > 0;
    }

    public function toArray(): array
    {
        return [
            'id' => $this->id,
            'uuid' => $this->uuid,
            'title' => $this->title,
            'slug' => $this->slug,
            'content' => $this->content,
            'excerpt' => $this->getGeneratedExcerpt(),
            'author_id' => $this->authorId,
            'status' => $this->status->value,
            'priority' => $this->priority->value,
            'published_at' => $this->publishedAt?->format('c'),
            'tags' => $this->tags,
            'metadata' => $this->metadata,
            'view_count' => $this->viewCount,
            'created_at' => $this->createdAt?->format('c'),
            'updated_at' => $this->updatedAt?->format('c'),
            'deleted_at' => $this->deletedAt?->format('c'),
        ];
    }
}

// ============================================================================
// COLLECTION CLASSES
// ============================================================================

/**
 * @template T
 * @implements IteratorAggregate<int, T>
 * @implements ArrayAccess<int, T>
 */
class Collection implements IteratorAggregate, ArrayAccess, Countable, JsonSerializable
{
    /** @var array<int, T> */
    protected array $items;

    /**
     * @param array<int, T> $items
     */
    public function __construct(array $items = [])
    {
        $this->items = array_values($items);
    }

    /**
     * @param iterable<T> $items
     * @return static
     */
    public static function from(iterable $items): static
    {
        return new static([...$items]);
    }

    /**
     * @template TValue
     * @param Closure(T): TValue $callback
     * @return Collection<TValue>
     */
    public function map(Closure $callback): Collection
    {
        return new Collection(array_map($callback, $this->items));
    }

    /**
     * @param Closure(T): bool $callback
     * @return static
     */
    public function filter(?Closure $callback = null): static
    {
        $items = $callback !== null
            ? array_filter($this->items, $callback)
            : array_filter($this->items);
        return new static(array_values($items));
    }

    /**
     * @param Closure(T): bool $callback
     * @return static
     */
    public function reject(Closure $callback): static
    {
        return $this->filter(fn($item) => !$callback($item));
    }

    /**
     * @template TAccumulator
     * @param Closure(TAccumulator, T): TAccumulator $callback
     * @param TAccumulator $initial
     * @return TAccumulator
     */
    public function reduce(Closure $callback, mixed $initial = null): mixed
    {
        return array_reduce($this->items, $callback, $initial);
    }

    /**
     * @param Closure(T): void $callback
     * @return static
     */
    public function each(Closure $callback): static
    {
        foreach ($this->items as $item) {
            if ($callback($item) === false) {
                break;
            }
        }
        return $this;
    }

    /**
     * @param Closure(T): bool|null $callback
     * @return T|null
     */
    public function first(?Closure $callback = null): mixed
    {
        if ($callback === null) {
            return $this->items[0] ?? null;
        }

        foreach ($this->items as $item) {
            if ($callback($item)) {
                return $item;
            }
        }

        return null;
    }

    /**
     * @param Closure(T): bool|null $callback
     * @return T|null
     */
    public function last(?Closure $callback = null): mixed
    {
        if ($callback === null) {
            return $this->items[array_key_last($this->items) ?? 0] ?? null;
        }

        $result = null;
        foreach ($this->items as $item) {
            if ($callback($item)) {
                $result = $item;
            }
        }

        return $result;
    }

    /**
     * @param T $value
     * @return bool
     */
    public function contains(mixed $value): bool
    {
        return in_array($value, $this->items, true);
    }

    /**
     * @param Closure(T): bool $callback
     * @return bool
     */
    public function some(Closure $callback): bool
    {
        foreach ($this->items as $item) {
            if ($callback($item)) {
                return true;
            }
        }
        return false;
    }

    /**
     * @param Closure(T): bool $callback
     * @return bool
     */
    public function every(Closure $callback): bool
    {
        foreach ($this->items as $item) {
            if (!$callback($item)) {
                return false;
            }
        }
        return true;
    }

    /**
     * @param Closure(T, T): int|null $callback
     * @return static
     */
    public function sort(?Closure $callback = null): static
    {
        $items = $this->items;
        if ($callback !== null) {
            usort($items, $callback);
        } else {
            sort($items);
        }
        return new static($items);
    }

    /**
     * @param string|Closure(T): mixed $key
     * @param string $direction
     * @return static
     */
    public function sortBy(string|Closure $key, string $direction = 'asc'): static
    {
        $items = $this->items;
        usort($items, function ($a, $b) use ($key, $direction) {
            $valueA = $key instanceof Closure ? $key($a) : $this->getValue($a, $key);
            $valueB = $key instanceof Closure ? $key($b) : $this->getValue($b, $key);
            $cmp = $valueA <=> $valueB;
            return strtolower($direction) === 'desc' ? -$cmp : $cmp;
        });
        return new static($items);
    }

    /**
     * @return static
     */
    public function reverse(): static
    {
        return new static(array_reverse($this->items));
    }

    /**
     * @return static
     */
    public function unique(): static
    {
        return new static(array_unique($this->items, SORT_REGULAR));
    }

    /**
     * @param string|Closure(T): mixed $key
     * @return static
     */
    public function uniqueBy(string|Closure $key): static
    {
        $seen = [];
        return $this->filter(function ($item) use ($key, &$seen) {
            $value = $key instanceof Closure ? $key($item) : $this->getValue($item, $key);
            if (in_array($value, $seen, true)) {
                return false;
            }
            $seen[] = $value;
            return true;
        });
    }

    /**
     * @param int $size
     * @return Collection<static>
     */
    public function chunk(int $size): Collection
    {
        return new Collection(
            array_map(
                fn($chunk) => new static($chunk),
                array_chunk($this->items, $size)
            )
        );
    }

    /**
     * @param int $depth
     * @return static
     */
    public function flatten(int $depth = PHP_INT_MAX): static
    {
        $result = [];
        $flatten = function (array $items, int $currentDepth) use (&$result, &$flatten, $depth): void {
            foreach ($items as $item) {
                if (is_array($item) && $currentDepth < $depth) {
                    $flatten($item, $currentDepth + 1);
                } elseif ($item instanceof self && $currentDepth < $depth) {
                    $flatten($item->items, $currentDepth + 1);
                } else {
                    $result[] = $item;
                }
            }
        };
        $flatten($this->items, 0);
        return new static($result);
    }

    /**
     * @param string|Closure(T): mixed $key
     * @return Collection<static>
     */
    public function groupBy(string|Closure $key): Collection
    {
        $groups = [];
        foreach ($this->items as $item) {
            $groupKey = $key instanceof Closure ? $key($item) : $this->getValue($item, $key);
            $groups[$groupKey][] = $item;
        }
        return new Collection(
            array_map(fn($group) => new static($group), $groups)
        );
    }

    /**
     * @param string|Closure(T): mixed $key
     * @return Collection<int|string, T>
     */
    public function keyBy(string|Closure $key): Collection
    {
        $result = [];
        foreach ($this->items as $item) {
            $itemKey = $key instanceof Closure ? $key($item) : $this->getValue($item, $key);
            $result[$itemKey] = $item;
        }
        return new Collection($result);
    }

    /**
     * @param string $key
     * @return static
     */
    public function pluck(string $key): static
    {
        return $this->map(fn($item) => $this->getValue($item, $key));
    }

    /**
     * @param string|Closure|null $key
     * @return int|float
     */
    public function sum(string|Closure|null $key = null): int|float
    {
        if ($key === null) {
            return array_sum($this->items);
        }

        return $this->reduce(function ($carry, $item) use ($key) {
            $value = $key instanceof Closure ? $key($item) : $this->getValue($item, $key);
            return $carry + $value;
        }, 0);
    }

    /**
     * @param string|Closure|null $key
     * @return int|float
     */
    public function avg(string|Closure|null $key = null): int|float
    {
        $count = count($this->items);
        return $count > 0 ? $this->sum($key) / $count : 0;
    }

    /**
     * @param string|Closure|null $key
     * @return mixed
     */
    public function min(string|Closure|null $key = null): mixed
    {
        if ($key === null) {
            return min($this->items);
        }

        $values = $this->map(fn($item) => $key instanceof Closure ? $key($item) : $this->getValue($item, $key));
        return min($values->items);
    }

    /**
     * @param string|Closure|null $key
     * @return mixed
     */
    public function max(string|Closure|null $key = null): mixed
    {
        if ($key === null) {
            return max($this->items);
        }

        $values = $this->map(fn($item) => $key instanceof Closure ? $key($item) : $this->getValue($item, $key));
        return max($values->items);
    }

    /**
     * @param int $start
     * @param int|null $length
     * @return static
     */
    public function slice(int $start, ?int $length = null): static
    {
        return new static(array_slice($this->items, $start, $length));
    }

    /**
     * @param int $count
     * @return static
     */
    public function take(int $count): static
    {
        return $this->slice(0, $count);
    }

    /**
     * @param int $count
     * @return static
     */
    public function skip(int $count): static
    {
        return $this->slice($count);
    }

    /**
     * @param static|array<T> $items
     * @return static
     */
    public function merge(self|array $items): static
    {
        $items = $items instanceof self ? $items->items : $items;
        return new static(array_merge($this->items, $items));
    }

    /**
     * @param static|array<T> $items
     * @return static
     */
    public function diff(self|array $items): static
    {
        $items = $items instanceof self ? $items->items : $items;
        return new static(array_values(array_diff($this->items, $items)));
    }

    /**
     * @param static|array<T> $items
     * @return static
     */
    public function intersect(self|array $items): static
    {
        $items = $items instanceof self ? $items->items : $items;
        return new static(array_values(array_intersect($this->items, $items)));
    }

    /**
     * @param T $item
     * @return static
     */
    public function push(mixed $item): static
    {
        $items = $this->items;
        $items[] = $item;
        return new static($items);
    }

    /**
     * @param T $item
     * @return static
     */
    public function prepend(mixed $item): static
    {
        return new static([$item, ...$this->items]);
    }

    /**
     * @return T|null
     */
    public function pop(): mixed
    {
        return array_pop($this->items);
    }

    /**
     * @return T|null
     */
    public function shift(): mixed
    {
        return array_shift($this->items);
    }

    /**
     * @param string $glue
     * @param string|Closure|null $key
     * @return string
     */
    public function implode(string $glue, string|Closure|null $key = null): string
    {
        if ($key === null) {
            return implode($glue, $this->items);
        }

        return $this->pluck($key)->implode($glue);
    }

    /**
     * @return bool
     */
    public function isEmpty(): bool
    {
        return empty($this->items);
    }

    /**
     * @return bool
     */
    public function isNotEmpty(): bool
    {
        return !$this->isEmpty();
    }

    /**
     * @return array<int, T>
     */
    public function toArray(): array
    {
        return $this->items;
    }

    /**
     * @return array<int, T>
     */
    public function all(): array
    {
        return $this->items;
    }

    public function count(): int
    {
        return count($this->items);
    }

    public function getIterator(): Traversable
    {
        return new ArrayIterator($this->items);
    }

    public function offsetExists(mixed $offset): bool
    {
        return isset($this->items[$offset]);
    }

    public function offsetGet(mixed $offset): mixed
    {
        return $this->items[$offset] ?? null;
    }

    public function offsetSet(mixed $offset, mixed $value): void
    {
        if ($offset === null) {
            $this->items[] = $value;
        } else {
            $this->items[$offset] = $value;
        }
    }

    public function offsetUnset(mixed $offset): void
    {
        unset($this->items[$offset]);
    }

    public function jsonSerialize(): array
    {
        return $this->items;
    }

    /**
     * @param T $item
     * @param string $key
     * @return mixed
     */
    protected function getValue(mixed $item, string $key): mixed
    {
        if (is_array($item)) {
            return $item[$key] ?? null;
        }

        if (is_object($item)) {
            return $item->{$key} ?? null;
        }

        return null;
    }
}

// ============================================================================
// UTILITY CLASSES
// ============================================================================

final class Str
{
    public static function lower(string $value): string
    {
        return mb_strtolower($value, 'UTF-8');
    }

    public static function upper(string $value): string
    {
        return mb_strtoupper($value, 'UTF-8');
    }

    public static function title(string $value): string
    {
        return mb_convert_case($value, MB_CASE_TITLE, 'UTF-8');
    }

    public static function camel(string $value): string
    {
        $value = ucwords(str_replace(['-', '_'], ' ', $value));
        return lcfirst(str_replace(' ', '', $value));
    }

    public static function pascal(string $value): string
    {
        return ucfirst(self::camel($value));
    }

    public static function snake(string $value, string $delimiter = '_'): string
    {
        $value = preg_replace('/\s+/u', '', ucwords($value));
        $value = preg_replace('/(.)(?=[A-Z])/u', '$1' . $delimiter, $value);
        return self::lower($value);
    }

    public static function kebab(string $value): string
    {
        return self::snake($value, '-');
    }

    public static function slug(string $value, string $separator = '-'): string
    {
        $value = self::lower($value);
        $value = preg_replace('/[^a-z0-9\s-]/u', '', $value);
        $value = preg_replace('/[\s-]+/', $separator, $value);
        return trim($value, $separator);
    }

    public static function startsWith(string $haystack, string|array $needles): bool
    {
        foreach ((array) $needles as $needle) {
            if (str_starts_with($haystack, $needle)) {
                return true;
            }
        }
        return false;
    }

    public static function endsWith(string $haystack, string|array $needles): bool
    {
        foreach ((array) $needles as $needle) {
            if (str_ends_with($haystack, $needle)) {
                return true;
            }
        }
        return false;
    }

    public static function contains(string $haystack, string|array $needles): bool
    {
        foreach ((array) $needles as $needle) {
            if (str_contains($haystack, $needle)) {
                return true;
            }
        }
        return false;
    }

    public static function length(string $value): int
    {
        return mb_strlen($value, 'UTF-8');
    }

    public static function limit(string $value, int $limit = 100, string $end = '...'): string
    {
        if (self::length($value) <= $limit) {
            return $value;
        }

        return rtrim(mb_substr($value, 0, $limit, 'UTF-8')) . $end;
    }

    public static function words(string $value, int $words = 100, string $end = '...'): string
    {
        preg_match('/^\s*+(?:\S++\s*+){1,' . $words . '}/u', $value, $matches);

        if (!isset($matches[0]) || self::length($value) === self::length($matches[0])) {
            return $value;
        }

        return rtrim($matches[0]) . $end;
    }

    public static function random(int $length = 16): string
    {
        $string = '';
        while (($len = strlen($string)) < $length) {
            $size = $length - $len;
            $bytes = random_bytes($size);
            $string .= substr(str_replace(['/', '+', '='], '', base64_encode($bytes)), 0, $size);
        }
        return $string;
    }

    public static function uuid(): string
    {
        return sprintf(
            '%04x%04x-%04x-%04x-%04x-%04x%04x%04x',
            mt_rand(0, 0xffff),
            mt_rand(0, 0xffff),
            mt_rand(0, 0xffff),
            mt_rand(0, 0x0fff) | 0x4000,
            mt_rand(0, 0x3fff) | 0x8000,
            mt_rand(0, 0xffff),
            mt_rand(0, 0xffff),
            mt_rand(0, 0xffff)
        );
    }

    public static function mask(string $value, string $character = '*', int $index = 0, ?int $length = null): string
    {
        $segment = mb_substr($value, $index, $length, 'UTF-8');
        $strlen = mb_strlen($segment, 'UTF-8');

        if ($strlen === 0) {
            return $value;
        }

        $start = mb_substr($value, 0, $index, 'UTF-8');
        $end = mb_substr($value, $index + $strlen, null, 'UTF-8');

        return $start . str_repeat($character, $strlen) . $end;
    }

    public static function padLeft(string $value, int $length, string $pad = ' '): string
    {
        $short = max(0, $length - mb_strlen($value, 'UTF-8'));
        return mb_substr(str_repeat($pad, $short), 0, $short, 'UTF-8') . $value;
    }

    public static function padRight(string $value, int $length, string $pad = ' '): string
    {
        $short = max(0, $length - mb_strlen($value, 'UTF-8'));
        return $value . mb_substr(str_repeat($pad, $short), 0, $short, 'UTF-8');
    }

    public static function repeat(string $value, int $times): string
    {
        return str_repeat($value, $times);
    }

    public static function replace(string|array $search, string|array $replace, string $subject): string
    {
        return str_replace($search, $replace, $subject);
    }

    public static function replaceFirst(string $search, string $replace, string $subject): string
    {
        $position = strpos($subject, $search);

        if ($position !== false) {
            return substr_replace($subject, $replace, $position, strlen($search));
        }

        return $subject;
    }

    public static function replaceLast(string $search, string $replace, string $subject): string
    {
        $position = strrpos($subject, $search);

        if ($position !== false) {
            return substr_replace($subject, $replace, $position, strlen($search));
        }

        return $subject;
    }

    public static function reverse(string $value): string
    {
        $result = '';
        $length = mb_strlen($value, 'UTF-8');
        for ($i = $length - 1; $i >= 0; $i--) {
            $result .= mb_substr($value, $i, 1, 'UTF-8');
        }
        return $result;
    }

    public static function substr(string $value, int $start, ?int $length = null): string
    {
        return mb_substr($value, $start, $length, 'UTF-8');
    }

    public static function substrCount(string $haystack, string $needle): int
    {
        return substr_count($haystack, $needle);
    }

    public static function trim(string $value, ?string $characters = null): string
    {
        return $characters !== null
            ? trim($value, $characters)
            : trim($value);
    }

    public static function ltrim(string $value, ?string $characters = null): string
    {
        return $characters !== null
            ? ltrim($value, $characters)
            : ltrim($value);
    }

    public static function rtrim(string $value, ?string $characters = null): string
    {
        return $characters !== null
            ? rtrim($value, $characters)
            : rtrim($value);
    }

    public static function split(string $value, string $delimiter): array
    {
        return explode($delimiter, $value);
    }

    public static function wrap(string $value, string $before, ?string $after = null): string
    {
        return $before . $value . ($after ?? $before);
    }

    public static function wordWrap(string $value, int $width = 75, string $break = "\n", bool $cutLongWords = false): string
    {
        return wordwrap($value, $width, $break, $cutLongWords);
    }

    public static function isUuid(string $value): bool
    {
        return preg_match('/^[\da-f]{8}-[\da-f]{4}-[\da-f]{4}-[\da-f]{4}-[\da-f]{12}$/iD', $value) === 1;
    }

    public static function isEmail(string $value): bool
    {
        return filter_var($value, FILTER_VALIDATE_EMAIL) !== false;
    }

    public static function isUrl(string $value): bool
    {
        return filter_var($value, FILTER_VALIDATE_URL) !== false;
    }

    public static function isJson(string $value): bool
    {
        json_decode($value);
        return json_last_error() === JSON_ERROR_NONE;
    }
}

final class Arr
{
    public static function get(array $array, string|int|null $key, mixed $default = null): mixed
    {
        if ($key === null) {
            return $array;
        }

        if (array_key_exists($key, $array)) {
            return $array[$key];
        }

        if (!str_contains((string) $key, '.')) {
            return $array[$key] ?? $default;
        }

        foreach (explode('.', (string) $key) as $segment) {
            if (!is_array($array) || !array_key_exists($segment, $array)) {
                return $default;
            }
            $array = $array[$segment];
        }

        return $array;
    }

    public static function set(array &$array, string|int|null $key, mixed $value): array
    {
        if ($key === null) {
            return $array = $value;
        }

        $keys = explode('.', (string) $key);
        $current = &$array;

        foreach ($keys as $i => $segment) {
            if (count($keys) === 1) {
                break;
            }

            unset($keys[$i]);

            if (!isset($current[$segment]) || !is_array($current[$segment])) {
                $current[$segment] = [];
            }

            $current = &$current[$segment];
        }

        $current[array_shift($keys)] = $value;

        return $array;
    }

    public static function has(array $array, string|array $keys): bool
    {
        $keys = (array) $keys;

        if (empty($keys)) {
            return false;
        }

        foreach ($keys as $key) {
            $subArray = $array;

            if (array_key_exists($key, $array)) {
                continue;
            }

            foreach (explode('.', $key) as $segment) {
                if (!is_array($subArray) || !array_key_exists($segment, $subArray)) {
                    return false;
                }
                $subArray = $subArray[$segment];
            }
        }

        return true;
    }

    public static function forget(array &$array, string|array $keys): void
    {
        $keys = (array) $keys;

        foreach ($keys as $key) {
            if (array_key_exists($key, $array)) {
                unset($array[$key]);
                continue;
            }

            $parts = explode('.', $key);
            $current = &$array;

            while (count($parts) > 1) {
                $part = array_shift($parts);

                if (isset($current[$part]) && is_array($current[$part])) {
                    $current = &$current[$part];
                } else {
                    continue 2;
                }
            }

            unset($current[array_shift($parts)]);
        }
    }

    public static function only(array $array, array $keys): array
    {
        return array_intersect_key($array, array_flip($keys));
    }

    public static function except(array $array, array $keys): array
    {
        return array_diff_key($array, array_flip($keys));
    }

    public static function first(array $array, ?callable $callback = null, mixed $default = null): mixed
    {
        if ($callback === null) {
            return empty($array) ? $default : reset($array);
        }

        foreach ($array as $key => $value) {
            if ($callback($value, $key)) {
                return $value;
            }
        }

        return $default;
    }

    public static function last(array $array, ?callable $callback = null, mixed $default = null): mixed
    {
        if ($callback === null) {
            return empty($array) ? $default : end($array);
        }

        return self::first(array_reverse($array, true), $callback, $default);
    }

    public static function where(array $array, callable $callback): array
    {
        return array_filter($array, $callback, ARRAY_FILTER_USE_BOTH);
    }

    public static function flatten(array $array, int $depth = PHP_INT_MAX): array
    {
        $result = [];

        foreach ($array as $item) {
            if (!is_array($item)) {
                $result[] = $item;
            } elseif ($depth === 1) {
                $result = array_merge($result, array_values($item));
            } else {
                $result = array_merge($result, self::flatten($item, $depth - 1));
            }
        }

        return $result;
    }

    public static function wrap(mixed $value): array
    {
        if ($value === null) {
            return [];
        }

        return is_array($value) ? $value : [$value];
    }

    public static function isAssoc(array $array): bool
    {
        $keys = array_keys($array);
        return $keys !== array_keys($keys);
    }

    public static function isList(array $array): bool
    {
        return !self::isAssoc($array);
    }

    public static function random(array $array, ?int $number = null): mixed
    {
        $requested = $number ?? 1;
        $count = count($array);

        if ($requested > $count) {
            throw new InvalidArgumentException(
                "You requested {$requested} items, but there are only {$count} items available."
            );
        }

        if ($number === null) {
            return $array[array_rand($array)];
        }

        $keys = array_rand($array, $requested);
        $results = [];

        foreach ((array) $keys as $key) {
            $results[] = $array[$key];
        }

        return $results;
    }

    public static function shuffle(array $array): array
    {
        shuffle($array);
        return $array;
    }

    public static function pluck(array $array, string|int $value, ?string $key = null): array
    {
        $results = [];

        foreach ($array as $item) {
            $itemValue = is_object($item) ? $item->{$value} : $item[$value];

            if ($key === null) {
                $results[] = $itemValue;
            } else {
                $itemKey = is_object($item) ? $item->{$key} : $item[$key];
                $results[$itemKey] = $itemValue;
            }
        }

        return $results;
    }

    public static function groupBy(array $array, string|callable $groupBy): array
    {
        $results = [];

        foreach ($array as $key => $value) {
            $groupKey = is_callable($groupBy) ? $groupBy($value, $key) : $value[$groupBy];
            $results[$groupKey][] = $value;
        }

        return $results;
    }

    public static function keyBy(array $array, string|callable $keyBy): array
    {
        $results = [];

        foreach ($array as $key => $value) {
            $resolvedKey = is_callable($keyBy) ? $keyBy($value, $key) : $value[$keyBy];
            $results[$resolvedKey] = $value;
        }

        return $results;
    }

    public static function map(array $array, callable $callback): array
    {
        $keys = array_keys($array);
        $items = array_map($callback, $array, $keys);

        return array_combine($keys, $items);
    }

    public static function mapWithKeys(array $array, callable $callback): array
    {
        $result = [];

        foreach ($array as $key => $value) {
            $assoc = $callback($value, $key);
            foreach ($assoc as $mapKey => $mapValue) {
                $result[$mapKey] = $mapValue;
            }
        }

        return $result;
    }

    public static function collapse(array $array): array
    {
        $results = [];

        foreach ($array as $values) {
            if (!is_array($values)) {
                continue;
            }

            $results[] = $values;
        }

        return array_merge([], ...$results);
    }

    public static function crossJoin(array ...$arrays): array
    {
        $results = [[]];

        foreach ($arrays as $array) {
            $append = [];

            foreach ($results as $product) {
                foreach ($array as $item) {
                    $append[] = array_merge($product, [$item]);
                }
            }

            $results = $append;
        }

        return $results;
    }

    public static function divide(array $array): array
    {
        return [array_keys($array), array_values($array)];
    }

    public static function undot(array $array): array
    {
        $results = [];

        foreach ($array as $key => $value) {
            self::set($results, $key, $value);
        }

        return $results;
    }

    public static function dot(array $array, string $prepend = ''): array
    {
        $results = [];

        foreach ($array as $key => $value) {
            if (is_array($value) && !empty($value)) {
                $results = array_merge($results, self::dot($value, $prepend . $key . '.'));
            } else {
                $results[$prepend . $key] = $value;
            }
        }

        return $results;
    }

    public static function sort(array $array, callable|string|null $callback = null): array
    {
        if ($callback === null) {
            asort($array);
            return $array;
        }

        if (is_string($callback)) {
            $callback = fn($a, $b) => ($a[$callback] ?? null) <=> ($b[$callback] ?? null);
        }

        uasort($array, $callback);

        return $array;
    }

    public static function sortDesc(array $array, callable|string|null $callback = null): array
    {
        if ($callback === null) {
            arsort($array);
            return $array;
        }

        if (is_string($callback)) {
            $callback = fn($a, $b) => ($b[$callback] ?? null) <=> ($a[$callback] ?? null);
        }

        uasort($array, fn($a, $b) => -$callback($a, $b));

        return $array;
    }

    public static function query(array $array): string
    {
        return http_build_query($array, '', '&', PHP_QUERY_RFC3986);
    }

    public static function toCssClasses(array $array): string
    {
        $classes = [];

        foreach ($array as $class => $constraint) {
            if (is_numeric($class)) {
                $classes[] = $constraint;
            } elseif ($constraint) {
                $classes[] = $class;
            }
        }

        return implode(' ', $classes);
    }

    public static function toCssStyles(array $array): string
    {
        $styles = [];

        foreach ($array as $property => $value) {
            if ($value !== null && $value !== false) {
                $styles[] = "{$property}: {$value}";
            }
        }

        return implode('; ', $styles);
    }
}

// ============================================================================
// FUNCTIONS
// ============================================================================

function collect(iterable $items = []): Collection
{
    return Collection::from($items);
}

function value(mixed $value, mixed ...$args): mixed
{
    return $value instanceof Closure ? $value(...$args) : $value;
}

function tap(mixed $value, ?callable $callback = null): mixed
{
    if ($callback !== null) {
        $callback($value);
    }

    return $value;
}

function retry(int $times, callable $callback, int $sleepMilliseconds = 0, ?callable $when = null): mixed
{
    $attempts = 0;
    $backoff = $sleepMilliseconds;

    beginning:
    $attempts++;

    try {
        return $callback($attempts);
    } catch (Throwable $e) {
        if ($attempts >= $times || ($when !== null && !$when($e))) {
            throw $e;
        }

        if ($backoff > 0) {
            usleep($backoff * 1000);
        }

        goto beginning;
    }
}

function once(callable $callback): mixed
{
    static $results = [];

    $hash = spl_object_hash((object) $callback);

    if (!isset($results[$hash])) {
        $results[$hash] = $callback();
    }

    return $results[$hash];
}

function env(string $key, mixed $default = null): mixed
{
    $value = $_ENV[$key] ?? getenv($key);

    if ($value === false) {
        return value($default);
    }

    return match (strtolower($value)) {
        'true', '(true)' => true,
        'false', '(false)' => false,
        'empty', '(empty)' => '',
        'null', '(null)' => null,
        default => $value,
    };
}

function throw_if(mixed $condition, Throwable|string $exception, mixed ...$parameters): mixed
{
    if ($condition) {
        if (is_string($exception)) {
            $exception = new RuntimeException($exception);
        }

        throw $exception;
    }

    return $condition;
}

function throw_unless(mixed $condition, Throwable|string $exception, mixed ...$parameters): mixed
{
    throw_if(!$condition, $exception, ...$parameters);

    return $condition;
}

function rescue(callable $callback, mixed $rescue = null, bool $report = true): mixed
{
    try {
        return $callback();
    } catch (Throwable $e) {
        if ($report) {
            // Log or report the error
        }

        return value($rescue, $e);
    }
}

function class_basename(string|object $class): string
{
    $class = is_object($class) ? get_class($class) : $class;

    return basename(str_replace('\\', '/', $class));
}

function class_uses_recursive(string|object $class): array
{
    if (is_object($class)) {
        $class = get_class($class);
    }

    $results = [];

    foreach (array_reverse(class_parents($class) ?: []) + [$class => $class] as $class) {
        $results += trait_uses_recursive($class);
    }

    return array_unique($results);
}

function trait_uses_recursive(string $trait): array
{
    $traits = class_uses($trait) ?: [];

    foreach ($traits as $trait) {
        $traits += trait_uses_recursive($trait);
    }

    return $traits;
}

function blank(mixed $value): bool
{
    if ($value === null) {
        return true;
    }

    if (is_string($value)) {
        return trim($value) === '';
    }

    if (is_numeric($value) || is_bool($value)) {
        return false;
    }

    if ($value instanceof Countable) {
        return count($value) === 0;
    }

    return empty($value);
}

function filled(mixed $value): bool
{
    return !blank($value);
}

function transform(mixed $value, callable $callback, mixed $default = null): mixed
{
    if (filled($value)) {
        return $callback($value);
    }

    return value($default);
}

function with(mixed $value, ?callable $callback = null): mixed
{
    return $callback !== null ? $callback($value) : $value;
}

function data_get(mixed $target, string|array|int|null $key, mixed $default = null): mixed
{
    if ($key === null) {
        return $target;
    }

    $key = is_array($key) ? $key : explode('.', (string) $key);

    foreach ($key as $segment) {
        if ($segment === '*') {
            if (!is_iterable($target)) {
                return value($default);
            }

            $result = [];
            foreach ($target as $item) {
                $result[] = data_get($item, $key);
            }

            return in_array('*', $key) ? Arr::collapse($result) : $result;
        }

        if (is_array($target) && array_key_exists($segment, $target)) {
            $target = $target[$segment];
        } elseif (is_object($target) && isset($target->{$segment})) {
            $target = $target->{$segment};
        } else {
            return value($default);
        }

        array_shift($key);
    }

    return $target;
}

function data_set(mixed &$target, string|array|int|null $key, mixed $value, bool $overwrite = true): mixed
{
    $segments = is_array($key) ? $key : explode('.', (string) $key);

    if (($segment = array_shift($segments)) === '*') {
        if (!is_array($target)) {
            $target = [];
        }

        if ($segments) {
            foreach ($target as &$inner) {
                data_set($inner, $segments, $value, $overwrite);
            }
        } elseif ($overwrite) {
            foreach ($target as &$inner) {
                $inner = $value;
            }
        }
    } elseif (is_array($target)) {
        if ($segments) {
            if (!isset($target[$segment])) {
                $target[$segment] = [];
            }

            data_set($target[$segment], $segments, $value, $overwrite);
        } elseif ($overwrite || !isset($target[$segment])) {
            $target[$segment] = $value;
        }
    } elseif (is_object($target)) {
        if ($segments) {
            if (!isset($target->{$segment})) {
                $target->{$segment} = [];
            }

            data_set($target->{$segment}, $segments, $value, $overwrite);
        } elseif ($overwrite || !isset($target->{$segment})) {
            $target->{$segment} = $value;
        }
    } else {
        $target = [];

        if ($segments) {
            data_set($target[$segment], $segments, $value, $overwrite);
        } elseif ($overwrite) {
            $target[$segment] = $value;
        }
    }

    return $target;
}

function data_forget(array &$target, string|array|int|null $key): void
{
    $segments = is_array($key) ? $key : explode('.', (string) $key);

    if (($segment = array_shift($segments)) === '*' && is_array($target)) {
        if ($segments) {
            foreach ($target as &$inner) {
                data_forget($inner, $segments);
            }
        }
    } elseif (is_array($target)) {
        if ($segments && isset($target[$segment])) {
            data_forget($target[$segment], $segments);
        } else {
            unset($target[$segment]);
        }
    }
}

// ============================================================================
// STUB CLASSES (for type hints)
// ============================================================================

interface ConnectionInterface {}
interface InputInterface {
    public function getArgument(string $name): mixed;
    public function getOption(string $name): mixed;
}
interface OutputInterface {
    public function writeln(string $message): void;
    public function error(string $message): void;
    public function confirm(string $question, bool $default = false): bool;
    public function ask(string $question, ?string $default = null): ?string;
    public function table(array $headers, array $rows): void;
    public function progressStart(int $max): void;
    public function progressAdvance(int $step = 1): void;
    public function progressFinish(): void;
}
interface ResponseInterface {}
interface ConstraintViolationListInterface {}
class Response {
    public function __construct(mixed $content = '', int $status = 200, array $headers = []) {}
}
class JsonResponse extends Response {}
class RedirectResponse extends Response {}
class Request {
    public function json(): array { return []; }
}
class Envelope {}
class NotFoundException extends RuntimeException {}
class ForbiddenException extends RuntimeException {}
class BadRequestException extends RuntimeException {}
class UserCreatedEvent extends AbstractEvent {
    public function __construct(public readonly User $user) {}
}
