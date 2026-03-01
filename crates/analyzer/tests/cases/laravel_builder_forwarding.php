<?php

declare(strict_types=1);

// ─── Laravel stub classes ────────────────────────────────────────────────────
// Minimal stubs for Builder forwarding tests (Phase 4).
// Only defines what's needed: Model, Eloquent\Builder, Query\Builder.

namespace Illuminate\Database\Eloquent {

    abstract class Model
    {
        protected array $casts = [];
        protected array $attributes = [];
        protected array $fillable = [];
        protected array $guarded = ['*'];
        protected array $hidden = [];
        protected string $table = '';
        protected string $primaryKey = 'id';
        protected string $keyType = 'int';
        public bool $incrementing = true;
        public bool $exists = false;
        public bool $wasRecentlyCreated = false;
        protected bool $timestamps = true;

        public function __construct(array $_attributes = [])
        {
        }

        public function __get(string $_name): mixed
        {
            return null;
        }

        public function __set(string $_name, mixed $_value): void
        {
        }

        public function __call(string $_method, array $_parameters): mixed
        {
            return null;
        }

        public static function __callStatic(string $_method, array $_parameters): mixed
        {
            return null;
        }
    }

    class Builder
    {
        protected mixed $model = null;
        protected mixed $query = null;

        public function __construct()
        {
        }

        /**
         * @return static
         */
        public function where(string $_column, mixed $_operator = null, mixed $_value = null): static
        {
            return $this;
        }

        /**
         * @return static
         */
        public function whereIn(string $_column, array $_values): static
        {
            return $this;
        }

        /**
         * @return static
         */
        public function orderBy(string $_column, string $_direction = 'asc'): static
        {
            return $this;
        }

        /**
         * @return static
         */
        public function limit(int $_value): static
        {
            return $this;
        }

        public function get(): Collection
        {
            return new Collection();
        }

        public function first(): ?Model
        {
            return null;
        }

        public function count(): int
        {
            return 0;
        }

        public function exists(): bool
        {
            return false;
        }

        /**
         * @return static
         */
        public function with(string|array $_relations): static
        {
            return $this;
        }
    }

    class Collection
    {
        public function __construct()
        {
        }
    }
}

namespace Illuminate\Database\Query {

    class Builder
    {
        public function __construct()
        {
        }

        /**
         * @return static
         */
        public function where(string $_column, mixed $_operator = null, mixed $_value = null): static
        {
            return $this;
        }

        /**
         * @return static
         */
        public function whereRaw(string $_sql, array $_bindings = []): static
        {
            return $this;
        }

        public function toSql(): string
        {
            return '';
        }
    }
}

// ─── Application models ─────────────────────────────────────────────────────

namespace App\Models {

    use Illuminate\Database\Eloquent\Model;
    use Illuminate\Database\Eloquent\Builder;

    class User extends Model
    {
        protected array $fillable = ['name', 'email'];

        protected array $casts = [
            'is_admin' => 'boolean',
        ];

        public function scopeActive(Builder $_query): Builder
        {
            return new Builder();
        }

        public function scopeOfType(Builder $_query, string $_type): Builder
        {
            return new Builder();
        }
    }

    class Post extends Model
    {
        protected array $fillable = ['title', 'body', 'user_id'];
    }
}

// ─── Test functions ──────────────────────────────────────────────────────────
// Tests that static method calls on Model subclasses are forwarded to Builder
// without producing errors (non-existent-method, non-existent-static-method, etc).
//
// The BuilderForwardingHook (StaticMethodCallHook) intercepts these calls and
// returns the mapped Builder return type.

namespace Tests\Laravel\BuilderForwarding {

    use App\Models\Post;
    use App\Models\User;

    /**
     * Basic static call forwarding: User::where(...) should not produce errors.
     * The issue filter suppresses non-existent-method errors on Model subclasses,
     * and the BuilderForwardingHook provides the return type.
     */
    function test_static_where(User $user): void
    {
        // Static method call forwarded to Builder::where()
        // The hook should resolve this without errors.
        $result = User::where('active', '=', true);
    }

    /**
     * Static orderBy forwarding.
     */
    function test_static_order_by(): void
    {
        $result = User::orderBy('name');
    }

    /**
     * Static limit forwarding.
     */
    function test_static_limit(): void
    {
        $result = User::limit(10);
    }

    /**
     * Static with() forwarding.
     */
    function test_static_with(): void
    {
        $result = User::with('posts');
    }

    /**
     * Static call on a different model.
     */
    function test_static_on_post(): void
    {
        $result = Post::where('published', '=', true);
    }

    /**
     * Scope methods should be callable statically on the model.
     * User::active() should resolve via BuilderForwardingHook's scope
     * check (which takes priority over builder method forwarding).
     */
    function test_static_scope_call(): void
    {
        $result = User::active();
    }

    /**
     * Scope with parameters.
     */
    function test_static_scope_with_params(): void
    {
        $result = User::ofType('admin');
    }

    /**
     * Real static property access on Model — should still work normally.
     */
    function test_real_model_properties(User $user): void
    {
        $exists = $user->exists;
    }
}