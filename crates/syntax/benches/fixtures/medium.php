<?php

declare(strict_types=1);

namespace Benchmark\Medium;

use Attribute;
use Closure;
use Exception;
use InvalidArgumentException;
use RuntimeException;
use Stringable;
use Throwable;

#[Attribute(Attribute::TARGET_CLASS | Attribute::TARGET_METHOD)]
final class Route
{
    public function __construct(
        public readonly string $path,
        public readonly string $method = 'GET',
        public readonly array $middleware = [],
    ) {}
}

#[Attribute(Attribute::TARGET_PROPERTY)]
final class Inject
{
    public function __construct(
        public readonly ?string $service = null,
    ) {}
}

interface RepositoryInterface
{
    public function find(int $id): ?object;
    public function findAll(): array;
    public function save(object $entity): void;
    public function delete(object $entity): void;
}

interface CacheInterface
{
    public function get(string $key, mixed $default = null): mixed;
    public function set(string $key, mixed $value, ?int $ttl = null): bool;
    public function has(string $key): bool;
    public function delete(string $key): bool;
}

trait TimestampTrait
{
    private ?\DateTimeImmutable $createdAt = null;
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

    public function setUpdatedAt(\DateTimeImmutable $updatedAt): static
    {
        $this->updatedAt = $updatedAt;
        return $this;
    }
}

enum Status: string
{
    case Pending = 'pending';
    case Active = 'active';
    case Inactive = 'inactive';
    case Deleted = 'deleted';

    public function label(): string
    {
        return match ($this) {
            self::Pending => 'Pending Approval',
            self::Active => 'Active',
            self::Inactive => 'Inactive',
            self::Deleted => 'Deleted',
        };
    }

    public function isEditable(): bool
    {
        return match ($this) {
            self::Pending, self::Active, self::Inactive => true,
            self::Deleted => false,
        };
    }
}

enum Priority: int
{
    case Low = 1;
    case Medium = 2;
    case High = 3;
    case Critical = 4;
}

abstract class AbstractEntity implements Stringable
{
    use TimestampTrait;

    protected ?int $id = null;

    public function getId(): ?int
    {
        return $this->id;
    }

    abstract public function validate(): bool;

    public function __toString(): string
    {
        return static::class . '#' . ($this->id ?? 'new');
    }
}

#[Route('/users', method: 'GET')]
class UserController
{
    #[Inject]
    private RepositoryInterface $repository;

    #[Inject(service: 'cache.users')]
    private CacheInterface $cache;

    public function __construct(
        private readonly LoggerInterface $logger,
        private readonly EventDispatcher $events,
    ) {}

    #[Route('/users/{id}', method: 'GET')]
    public function show(int $id): Response
    {
        $cacheKey = "user:{$id}";

        if ($this->cache->has($cacheKey)) {
            return new Response($this->cache->get($cacheKey));
        }

        $user = $this->repository->find($id);

        if ($user === null) {
            throw new NotFoundException("User {$id} not found");
        }

        $this->cache->set($cacheKey, $user, ttl: 3600);

        return new Response($user);
    }

    #[Route('/users', method: 'POST')]
    public function create(Request $request): Response
    {
        $data = $request->json();

        $user = new User(
            name: $data['name'] ?? throw new InvalidArgumentException('Name is required'),
            email: $data['email'] ?? throw new InvalidArgumentException('Email is required'),
            status: Status::Pending,
        );

        try {
            $this->repository->save($user);
            $this->events->dispatch(new UserCreatedEvent($user));
            $this->logger->info('User created', ['id' => $user->getId()]);
        } catch (Throwable $e) {
            $this->logger->error('Failed to create user', [
                'error' => $e->getMessage(),
                'trace' => $e->getTraceAsString(),
            ]);
            throw new RuntimeException('Failed to create user', previous: $e);
        }

        return new Response($user, status: 201);
    }

    #[Route('/users/{id}', method: 'DELETE')]
    public function delete(int $id): Response
    {
        $user = $this->repository->find($id)
            ?? throw new NotFoundException("User {$id} not found");

        $this->repository->delete($user);
        $this->cache->delete("user:{$id}");

        return new Response(null, status: 204);
    }
}

class User extends AbstractEntity
{
    public function __construct(
        private string $name,
        private string $email,
        private Status $status = Status::Active,
        private Priority $priority = Priority::Medium,
        private array $roles = [],
        private array $metadata = [],
    ) {
        $this->createdAt = new \DateTimeImmutable();
    }

    public function getName(): string
    {
        return $this->name;
    }

    public function setName(string $name): static
    {
        $this->name = $name;
        $this->updatedAt = new \DateTimeImmutable();
        return $this;
    }

    public function getEmail(): string
    {
        return $this->email;
    }

    public function setEmail(string $email): static
    {
        if (!filter_var($email, FILTER_VALIDATE_EMAIL)) {
            throw new InvalidArgumentException("Invalid email: {$email}");
        }
        $this->email = $email;
        $this->updatedAt = new \DateTimeImmutable();
        return $this;
    }

    public function getStatus(): Status
    {
        return $this->status;
    }

    public function setStatus(Status $status): static
    {
        $this->status = $status;
        $this->updatedAt = new \DateTimeImmutable();
        return $this;
    }

    public function getPriority(): Priority
    {
        return $this->priority;
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

    public function hasRole(string $role): bool
    {
        return in_array($role, $this->roles, true);
    }

    public function getMetadata(): array
    {
        return $this->metadata;
    }

    public function setMetadata(string $key, mixed $value): static
    {
        $this->metadata[$key] = $value;
        return $this;
    }

    public function validate(): bool
    {
        return !empty($this->name)
            && !empty($this->email)
            && filter_var($this->email, FILTER_VALIDATE_EMAIL) !== false;
    }

    public function toArray(): array
    {
        return [
            'id' => $this->id,
            'name' => $this->name,
            'email' => $this->email,
            'status' => $this->status->value,
            'priority' => $this->priority->value,
            'roles' => $this->roles,
            'metadata' => $this->metadata,
            'created_at' => $this->createdAt?->format('c'),
            'updated_at' => $this->updatedAt?->format('c'),
        ];
    }
}

class Collection implements \IteratorAggregate, \Countable
{
    private array $items;

    public function __construct(array $items = [])
    {
        $this->items = $items;
    }

    public static function from(iterable $items): static
    {
        return new static([...$items]);
    }

    public function map(Closure $callback): static
    {
        return new static(array_map($callback, $this->items));
    }

    public function filter(?Closure $callback = null): static
    {
        return new static(
            $callback !== null
                ? array_filter($this->items, $callback)
                : array_filter($this->items)
        );
    }

    public function reduce(Closure $callback, mixed $initial = null): mixed
    {
        return array_reduce($this->items, $callback, $initial);
    }

    public function first(?Closure $callback = null): mixed
    {
        if ($callback === null) {
            return $this->items[array_key_first($this->items)] ?? null;
        }

        foreach ($this->items as $item) {
            if ($callback($item)) {
                return $item;
            }
        }

        return null;
    }

    public function last(?Closure $callback = null): mixed
    {
        if ($callback === null) {
            return $this->items[array_key_last($this->items)] ?? null;
        }

        $result = null;
        foreach ($this->items as $item) {
            if ($callback($item)) {
                $result = $item;
            }
        }

        return $result;
    }

    public function contains(mixed $value): bool
    {
        return in_array($value, $this->items, true);
    }

    public function unique(): static
    {
        return new static(array_unique($this->items, SORT_REGULAR));
    }

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

    public function reverse(): static
    {
        return new static(array_reverse($this->items));
    }

    public function chunk(int $size): static
    {
        return new static(array_chunk($this->items, $size));
    }

    public function flatten(int $depth = PHP_INT_MAX): static
    {
        $result = [];
        $flatten = function (array $items, int $currentDepth) use (&$result, &$flatten, $depth): void {
            foreach ($items as $item) {
                if (is_array($item) && $currentDepth < $depth) {
                    $flatten($item, $currentDepth + 1);
                } else {
                    $result[] = $item;
                }
            }
        };
        $flatten($this->items, 0);
        return new static($result);
    }

    public function groupBy(string|Closure $key): static
    {
        $groups = [];
        foreach ($this->items as $item) {
            $groupKey = $key instanceof Closure
                ? $key($item)
                : (is_object($item) ? $item->{$key} : $item[$key]);
            $groups[$groupKey][] = $item;
        }
        return new static($groups);
    }

    public function pluck(string $key): static
    {
        return $this->map(fn($item) => is_object($item) ? $item->{$key} : $item[$key]);
    }

    public function sum(string|Closure|null $key = null): int|float
    {
        if ($key === null) {
            return array_sum($this->items);
        }

        return $this->reduce(function ($carry, $item) use ($key) {
            $value = $key instanceof Closure
                ? $key($item)
                : (is_object($item) ? $item->{$key} : $item[$key]);
            return $carry + $value;
        }, 0);
    }

    public function avg(string|Closure|null $key = null): int|float
    {
        $count = count($this->items);
        return $count > 0 ? $this->sum($key) / $count : 0;
    }

    public function min(string|Closure|null $key = null): mixed
    {
        if ($key === null) {
            return min($this->items);
        }

        return $this->reduce(function ($carry, $item) use ($key) {
            $value = $key instanceof Closure
                ? $key($item)
                : (is_object($item) ? $item->{$key} : $item[$key]);
            return $carry === null || $value < $carry ? $value : $carry;
        });
    }

    public function max(string|Closure|null $key = null): mixed
    {
        if ($key === null) {
            return max($this->items);
        }

        return $this->reduce(function ($carry, $item) use ($key) {
            $value = $key instanceof Closure
                ? $key($item)
                : (is_object($item) ? $item->{$key} : $item[$key]);
            return $carry === null || $value > $carry ? $value : $carry;
        });
    }

    public function isEmpty(): bool
    {
        return empty($this->items);
    }

    public function isNotEmpty(): bool
    {
        return !$this->isEmpty();
    }

    public function toArray(): array
    {
        return $this->items;
    }

    public function count(): int
    {
        return count($this->items);
    }

    public function getIterator(): \Traversable
    {
        return new \ArrayIterator($this->items);
    }
}

function process_data(array $input): array
{
    $collection = Collection::from($input);

    return $collection
        ->filter(fn($item) => $item['active'] ?? false)
        ->map(fn($item) => [
            ...$item,
            'processed' => true,
            'processed_at' => date('c'),
        ])
        ->sort(fn($a, $b) => ($a['priority'] ?? 0) <=> ($b['priority'] ?? 0))
        ->toArray();
}

function calculate_statistics(Collection $data): array
{
    $values = $data->pluck('value');

    return [
        'count' => $values->count(),
        'sum' => $values->sum(),
        'avg' => $values->avg(),
        'min' => $values->min(),
        'max' => $values->max(),
        'range' => $values->max() - $values->min(),
    ];
}
