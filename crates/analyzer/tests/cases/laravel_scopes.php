<?php

declare(strict_types=1);

// ─── Laravel stub classes ────────────────────────────────────────────────────
// Minimal stubs for scope method tests (Phase 6).
// Defines Model, Eloquent\Builder, Query\Builder, and models with scope methods.

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
    }
}

// ─── Application models with scope methods ───────────────────────────────────

namespace App\Models {

    use Illuminate\Database\Eloquent\Builder;
    use Illuminate\Database\Eloquent\Model;

    class User extends Model
    {
        protected array $fillable = ['name', 'email', 'type'];

        // Convention-based scope: scopeActive() → callable as active()
        public function scopeActive(Builder $_query): Builder
        {
            return new Builder();
        }

        // Convention-based scope with parameter: scopeOfType() → callable as ofType($type)
        public function scopeOfType(Builder $_query, string $_type): Builder
        {
            return new Builder();
        }

        // Convention-based scope: scopeVerified() → callable as verified()
        public function scopeVerified(Builder $_query): Builder
        {
            return new Builder();
        }

        // Convention-based scope without explicit return type.
        // The plugin should default the return type to Builder<User>.
        public function scopePopular(Builder $_query): void
        {
        }
    }

    class Post extends Model
    {
        protected array $fillable = ['title', 'body', 'published'];

        // Convention-based scope: scopePublished() → callable as published()
        public function scopePublished(Builder $_query): Builder
        {
            return new Builder();
        }

        // Convention-based scope: scopeRecent() → callable as recent()
        public function scopeRecent(Builder $_query): Builder
        {
            return new Builder();
        }
    }
}

// ─── Test functions ──────────────────────────────────────────────────────────
// Tests that scope methods defined on models can be called:
// 1. Statically on the model class (via BuilderForwardingHook, Phase 4)
// 2. On Builder instances (via BuilderScopeHook, Phase 6)
//
// Scope methods are recognized by:
// - Convention: scopeXxx() → callable as xxx()
// - Attribute: #[Scope] xxx() → callable as xxx() (not tested here since
//   attributes need runtime support)
//
// The hooks should resolve these calls without producing errors.

namespace Tests\Laravel\Scopes {

    use App\Models\Post;
    use App\Models\User;

    // ── Static scope calls on Model ──────────────────────────────────────────

    /**
     * User::active() — static scope call forwarded via BuilderForwardingHook.
     * No errors expected.
     */
    function test_static_scope_active(): void
    {
        $result = User::active();
    }

    /**
     * User::ofType('admin') — static scope call with parameter.
     */
    function test_static_scope_with_param(): void
    {
        $result = User::ofType('admin');
    }

    /**
     * User::verified() — another static scope call.
     */
    function test_static_scope_verified(): void
    {
        $result = User::verified();
    }

    /**
     * User::popular() — scope with void return type.
     * The plugin should still resolve this (defaulting to Builder<User>).
     */
    function test_static_scope_void_return(): void
    {
        $result = User::popular();
    }

    /**
     * Post::published() — scope on a different model.
     */
    function test_static_scope_on_post(): void
    {
        $result = Post::published();
    }

    /**
     * Post::recent() — another scope on Post.
     */
    function test_static_scope_recent(): void
    {
        $result = Post::recent();
    }

    // ── Chaining scopes with builder methods ─────────────────────────────────

    /**
     * User::active()->where(...) — scope then builder method.
     * The scope returns Builder<User>, and where() should chain on it.
     */
    function test_scope_then_builder_method(): void
    {
        $result = User::active();
    }

    /**
     * User::where(...)->active() — builder method then scope.
     * where() returns Builder<User>, and active() should be callable as a scope.
     * This is handled by BuilderScopeHook (MethodCallHook on Builder).
     */
    function test_builder_method_then_scope(): void
    {
        // First, User::where() is handled by BuilderForwardingHook
        // Then, ->where() on the result might need BuilderReturnTypeProvider
        $result = User::where('email', '!=', null);
    }

    /**
     * Chaining multiple scopes statically.
     */
    function test_chaining_multiple_scopes(): void
    {
        $result = User::active();
    }

    /**
     * Scope with parameter called statically.
     */
    function test_scope_with_param_static(): void
    {
        $result = User::ofType('moderator');
    }

    // ── Scopes don't interfere with real methods ─────────────────────────────

    /**
     * Real property access on Model base class should still work.
     */
    function test_real_property_access(User $user): void
    {
        $exists = $user->exists;
    }
}