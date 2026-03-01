<?php

declare(strict_types=1);

// ─── Laravel stub classes ────────────────────────────────────────────────────
// Minimal stubs for Factory return type tests (Phase 5).
// Only defines what's needed: Model, Factory base class, and concrete factories.

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
}

namespace Illuminate\Database\Eloquent\Factories {

    use Illuminate\Database\Eloquent\Model;

    abstract class Factory
    {
        protected string $model = '';
        protected ?int $count = null;
        protected array $states = [];
        protected array $afterMaking = [];
        protected array $afterCreating = [];

        /**
         * @return static
         */
        public function count(int $_count): static
        {
            return $this;
        }

        /**
         * @return static
         */
        public function state(array|callable $_state): static
        {
            return $this;
        }

        public function create(array $_attributes = []): mixed
        {
            return null;
        }

        public function make(array $_attributes = []): mixed
        {
            return null;
        }
    }
}

// ─── Application models ─────────────────────────────────────────────────────

namespace App\Models {

    use Illuminate\Database\Eloquent\Model;

    class User extends Model
    {
        protected array $fillable = ['name', 'email'];
    }

    class Post extends Model
    {
        protected array $fillable = ['title', 'body'];
    }
}

// ─── Application factories ───────────────────────────────────────────────────
// These follow the Laravel naming convention: Database\Factories\{Name}Factory
// The FactoryReturnTypeProvider derives the model FQN from the factory class name.

namespace Database\Factories {

    use Illuminate\Database\Eloquent\Factories\Factory;

    // Factory WITHOUT explicit @extends generics.
    // The FactoryReturnTypeProvider should derive the model from naming convention:
    // Database\Factories\UserFactory → App\Models\User
    class UserFactory extends Factory
    {
    }

    // Another factory without explicit generics.
    // Database\Factories\PostFactory → App\Models\Post
    class PostFactory extends Factory
    {
    }
}

// ─── Test functions ──────────────────────────────────────────────────────────
// Tests that Factory::create() and Factory::make() return the correct model type
// when the factory follows the naming convention.
//
// The FactoryReturnTypeProvider (MethodReturnTypeProvider) intercepts create/make
// on Factory subclasses and returns the model type derived from naming convention.

namespace Tests\Laravel\Factory {

    use App\Models\Post;
    use App\Models\User;
    use Database\Factories\PostFactory;
    use Database\Factories\UserFactory;

    // UserFactory::create() should return User (or at least a concrete type).
    // Without the plugin, it would return mixed (from Factory::create's native return type).
    function test_user_factory_create(): void
    {
        $factory = new UserFactory();
        $user = $factory->create();
    }

    // UserFactory::make() should return User.
    function test_user_factory_make(): void
    {
        $factory = new UserFactory();
        $user = $factory->make();
    }

    // PostFactory::create() should return Post.
    function test_post_factory_create(): void
    {
        $factory = new PostFactory();
        $post = $factory->create();
    }

    // PostFactory::make() should return Post.
    function test_post_factory_make(): void
    {
        $factory = new PostFactory();
        $post = $factory->make();
    }

    // Chaining state() before create() should still work.
    function test_factory_with_state(): void
    {
        $factory = new UserFactory();
        $user = $factory->state(['is_admin' => true])->create();
    }

    // Chaining count() before create() should still work.
    function test_factory_with_count(): void
    {
        $factory = new UserFactory();
        $user = $factory->count(5)->create();
    }
}