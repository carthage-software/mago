<?php

declare(strict_types=1);

// ─── Laravel stub classes ────────────────────────────────────────────────────
// Minimal stubs so the analyzer sees the class hierarchy and conventions.
// We keep stubs as simple as possible to avoid unrelated diagnostics.
// See laravel_cast_minimal.php for the template these stubs follow.

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
        protected string $dateFormat = '';
        protected string $connection = '';
        protected array $with = [];
        protected array $withCount = [];
        protected int $perPage = 15;
        protected array $appends = [];
        protected array $visible = [];
        protected array $touches = [];
        protected array $observables = [];
        protected array $relations = [];
        protected array $dates = [];
        protected array $dispatchesEvents = [];

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

    class Collection
    {
        public function __construct()
        {
        }
    }
}

namespace Illuminate\Support {
    class Carbon extends \DateTimeImmutable
    {
    }
}

namespace Illuminate\Database\Eloquent\Relations {

    use Illuminate\Database\Eloquent\Model;

    /**
     * @template TRelated of Model
     */
    abstract class Relation
    {
        /** @var TRelated */
        protected mixed $related = null;
    }

    /**
     * @template TRelated of Model
     * @extends Relation<TRelated>
     */
    class HasOne extends Relation
    {
        public function __construct()
        {
        }
    }

    /**
     * @template TRelated of Model
     * @extends Relation<TRelated>
     */
    class HasMany extends Relation
    {
        public function __construct()
        {
        }
    }

    /**
     * @template TRelated of Model
     * @extends Relation<TRelated>
     */
    class BelongsTo extends Relation
    {
        public function __construct()
        {
        }
    }

    /**
     * @template TRelated of Model
     * @extends Relation<TRelated>
     */
    class BelongsToMany extends Relation
    {
        public function __construct()
        {
        }
    }

    /**
     * @template TRelated of Model
     * @extends Relation<TRelated>
     */
    class MorphOne extends Relation
    {
        public function __construct()
        {
        }
    }

    /**
     * @template TRelated of Model
     * @extends Relation<TRelated>
     */
    class MorphMany extends Relation
    {
        public function __construct()
        {
        }
    }

    /**
     * @template TRelated of Model
     * @extends Relation<TRelated>
     */
    class MorphTo extends Relation
    {
        public function __construct()
        {
        }
    }

    /**
     * @template TRelated of Model
     * @extends Relation<TRelated>
     */
    class HasOneThrough extends Relation
    {
        public function __construct()
        {
        }
    }

    /**
     * @template TRelated of Model
     * @extends Relation<TRelated>
     */
    class HasManyThrough extends Relation
    {
        public function __construct()
        {
        }
    }
}

namespace Illuminate\Database\Eloquent\Casts {

    /**
     * @template TGet
     */
    class Attribute
    {
        /** @var TGet */
        public mixed $get = null;

        public function __construct(?callable $_get = null, ?callable $_set = null)
        {
        }
    }
}

// ─── Application models ─────────────────────────────────────────────────────

namespace App\Models {

    use Illuminate\Database\Eloquent\Casts\Attribute;
    use Illuminate\Database\Eloquent\Model;
    use Illuminate\Database\Eloquent\Relations\BelongsTo;
    use Illuminate\Database\Eloquent\Relations\BelongsToMany;
    use Illuminate\Database\Eloquent\Relations\HasMany;
    use Illuminate\Database\Eloquent\Relations\HasOne;
    use Illuminate\Database\Eloquent\Relations\MorphMany;
    use Illuminate\Database\Eloquent\Relations\MorphTo;

    class Post extends Model
    {
        protected array $casts = [
            'published_at' => 'datetime',
            'is_featured'  => 'boolean',
            'view_count'   => 'integer',
            'rating'       => 'float',
            'title'        => 'string',
        ];

        protected array $fillable = ['slug', 'body'];

        protected array $attributes = [
            'status' => 'draft',
        ];

        /**
         * @return BelongsTo<User>
         */
        public function author(): BelongsTo
        {
            /** @var BelongsTo<User> */
            return new BelongsTo();
        }

        /**
         * @return HasMany<Comment>
         */
        public function comments(): HasMany
        {
            /** @var HasMany<Comment> */
            return new HasMany();
        }

        /**
         * @return BelongsToMany<Tag>
         */
        public function tags(): BelongsToMany
        {
            /** @var BelongsToMany<Tag> */
            return new BelongsToMany();
        }

        /**
         * @return MorphMany<Image>
         */
        public function images(): MorphMany
        {
            /** @var MorphMany<Image> */
            return new MorphMany();
        }

        // Legacy accessor: getExcerptAttribute() → $excerpt
        public function getExcerptAttribute(): string
        {
            return 'excerpt';
        }

        // Modern accessor (Laravel 9+): formattedTitle() → $formatted_title
        /**
         * @return Attribute<string>
         */
        public function formattedTitle(): Attribute
        {
            /** @var Attribute<string> */
            return new Attribute(fn () => 'title');
        }
    }

    class User extends Model
    {
        protected array $fillable = ['name', 'email'];

        protected array $hidden = ['password'];

        /**
         * @return HasMany<Post>
         */
        public function posts(): HasMany
        {
            /** @var HasMany<Post> */
            return new HasMany();
        }

        /**
         * @return HasOne<Profile>
         */
        public function profile(): HasOne
        {
            /** @var HasOne<Profile> */
            return new HasOne();
        }

        // Legacy accessor: getFullNameAttribute() → $full_name
        public function getFullNameAttribute(): string
        {
            return 'full name';
        }
    }

    class Profile extends Model
    {
    }

    class Comment extends Model
    {
        /**
         * @return BelongsTo<Post>
         */
        public function post(): BelongsTo
        {
            /** @var BelongsTo<Post> */
            return new BelongsTo();
        }

        // MorphTo — no @return docblock needed; the plugin classifies it
        // from the native return type and always resolves to Model|null.
        // Using @return MorphTo<Model> would be redundant (Model is the
        // default bound for TRelated).
        public function commentable(): MorphTo
        {
            return new MorphTo();
        }
    }

    class Tag extends Model
    {
        /**
         * @return BelongsToMany<Post>
         */
        public function posts(): BelongsToMany
        {
            /** @var BelongsToMany<Post> */
            return new BelongsToMany();
        }
    }

    class Image extends Model
    {
    }
}

// ─── Test functions ──────────────────────────────────────────────────────────
// Each function accesses virtual properties on Eloquent models.
// The Laravel plugin's ExpressionHook resolves virtual property types from
// $casts, $attributes, $fillable/$hidden, accessors, and relationships.
//
// Cast/accessor/relationship properties get precise types (no issues).
// Column-name-only properties ($fillable/$hidden) resolve to `mixed`.

namespace Tests\Laravel\ModelProperties {

    use App\Models\Comment;
    use App\Models\Post;
    use App\Models\User;

    /**
     * Cast properties: $casts array entries should resolve to typed properties.
     * No mixed-assignment expected — the plugin provides precise types.
     */
    function test_cast_properties(Post $post): void
    {
        // 'published_at' => 'datetime' → Illuminate\Support\Carbon
        $publishedAt = $post->published_at;
        // 'is_featured' => 'boolean' → bool
        $isFeatured = $post->is_featured;
        // 'view_count' => 'integer' → int
        $viewCount = $post->view_count;
        // 'rating' => 'float' → float
        $rating = $post->rating;
        // 'title' => 'string' → string
        $title = $post->title;
    }

    /**
     * $fillable column names resolve to `mixed` (no specific type info).
     */
    function test_fillable_properties(Post $post): void
    {
        // @mago-expect analysis:mixed-assignment
        $slug = $post->slug;
        // @mago-expect analysis:mixed-assignment
        $body = $post->body;
    }

    /**
     * $attributes defaults: the plugin infers the type from the default value.
     */
    function test_attribute_defaults(Post $post): void
    {
        // 'status' => 'draft' → literal string type
        $status = $post->status;
    }

    /**
     * Relationship properties resolve to model or collection types.
     */
    function test_relationship_properties(Post $post): void
    {
        // BelongsTo (singular) → User|null
        $author = $post->author;

        // HasMany (collection) → Collection<Comment>
        $comments = $post->comments;

        // BelongsToMany (collection) → Collection<Tag>
        $tags = $post->tags;

        // MorphMany (collection) → Collection<Image>
        $images = $post->images;
    }

    /**
     * Relationship properties on User.
     */
    function test_user_relationships(User $user): void
    {
        // HasMany (collection) → Collection<Post>
        $posts = $user->posts;

        // HasOne (singular) → Profile|null
        $profile = $user->profile;
    }

    /**
     * Legacy accessor: getExcerptAttribute() → $excerpt (string).
     */
    function test_legacy_accessor(Post $post): void
    {
        $excerpt = $post->excerpt;
    }

    /**
     * Modern accessor (Laravel 9+): formattedTitle(): Attribute<string> → $formatted_title.
     */
    function test_modern_accessor(Post $post): void
    {
        $formattedTitle = $post->formatted_title;
    }

    /**
     * Legacy accessor on User: getFullNameAttribute() → $full_name.
     */
    function test_user_accessor(User $user): void
    {
        $fullName = $user->full_name;
    }

    /**
     * $fillable / $hidden column name properties on User.
     * These resolve to `mixed`, which means mixed-assignment.
     */
    function test_user_column_names(User $user): void
    {
        // @mago-expect analysis:mixed-assignment
        $name = $user->name;
        // @mago-expect analysis:mixed-assignment
        $email = $user->email;
        // @mago-expect analysis:mixed-assignment
        $password = $user->password;
    }

    /**
     * MorphTo relationship property → Model|null.
     */
    function test_morph_to(Comment $comment): void
    {
        $commentable = $comment->commentable;
    }

    /**
     * Relationship count properties: posts_count → int.
     */
    function test_count_properties(User $user): void
    {
        $postsCount = $user->posts_count;
    }

    /**
     * Real declared properties on Model base class should still work normally.
     */
    function test_real_framework_properties(User $user): void
    {
        $exists = $user->exists;
        $wasRecentlyCreated = $user->wasRecentlyCreated;
    }
}