<?php

final readonly class PlaygroundConstructorExamples
{
    public function __construct(
        private string                $from,
        private string                $to,
        private ?string               $hostId = null,
        private ?WhatsappLanguageEnum $language = null,
        private ?string               $body = null,
        private ?string               $media = null,
        private ?TemplateTypeEnum     $templateCode = null,
        private ?array                $variables = null,
        private ?array                $buttonVariables = null,
        private ?string               $mediaType = null,
        private ?string               $filename = null,
        private ?int                  $delay = null,
        private ?int                  $chatMessageId = null,
    ) {}

    public function withMixedParameters(
        private readonly FooBarBaz $foo,
        string                     $short,
        ?VeryLongTypeName          $optional = null,
        BarBazQux                  $qux,
    ): void {}

    public function containsAllPlaceholders(
        public array $placeholders = [],
        mixed        $options = null,
        ?array       $groups = null,
        mixed        $payload = null,
    ) {}

    public function exportGraphQlExamples(
        #[Autowire(service: 'api_platform.graphql.schema_builder')]
        private readonly object $schemaBuilder,
        private readonly string $projectDir,
        ?ClockInterface         $clock = null,
    ) {}

    public function auditLogUpload(
        private readonly EntityManagerInterface $em,
        private readonly LoggerInterface        $logger,
        string                                  $auditLogsS3Endpoint,
        string                                  $auditLogsS3Bucket,
        string                                  $auditLogsS3AccessKey,
        string                                  $auditLogsS3SecretKey,
        string                                  $auditLogsS3Region,
        private readonly AuditLogRepository     $auditLogRepository,
    ) {}

    public function multiFieldSearch(
        private readonly ?array $fields = null,
                                $properties = null,
        ?string                 $strategy = null,
    ) {}

    public function buildSefRequest(
        private readonly LoggerInterface $logger,
        ?string                          $schemaPath = null,
    ) {}

    public function whatsappApiException(
        private readonly int     $statusCode,
        private readonly ?int    $errorCode,
        private readonly ?int    $errorSubcode,
        private readonly bool    $transient,
        private readonly ?string $responseBody,
        string                   $reason,
        ?Throwable               $previous = null,
        private readonly ?string $step = null,
        private readonly ?string $method = null,
        private readonly ?string $url = null,
        private readonly ?string $errorUserTitle = null,
        private readonly ?string $errorUserMessage = null,
        private readonly ?string $fbtraceId = null,
        private readonly array   $context = [],
    ) {}

    public function commentedModifiers(
        /* keep comment */
        public readonly string $first,
        public /* inline block */ readonly int $second,
        private readonly /* multiline
         * block */ FooBar $third,
        protected string $fourth, // keep trailing line comment
        private readonly string /** after type */ $fifth,
    ) {}

    public function mixedWithPlainComments(
        private readonly FooBarBaz $foo,
        /** keep doc comment */
        public readonly string $first,
        string $short,
        ?VeryLongTypeName $optional = null,
        public /* inline block */ readonly int $second,
        BarBazQux $qux,
        private readonly /* multiline
         * block */ FooBar $third,
        protected string $fourth, // keep trailing line comment
        private readonly string /** after type */ $fifth,
    ): void {}

    public function __construct(
        private readonly ?array  $fields = null,
        private                  $values,
                                 $properties = null,
        ?string                  $strategy = null,
    ) {}

    public function filterProperty(
        string                      $property,
                                    $value,
        QueryBuilder                $queryBuilder,
        QueryNameGeneratorInterface $queryNameGenerator,
        string                      $resourceClass,
        ?Operation                  $operation = null,
        array                       $context = [],
    ): void {}

    public function filterPropertiesById(string $property, int $value, $queryBuilder, array $context = []): void {}

    public function filterPropertiesById2(
        string    $property,
        int       $value,
                  $queryBuilder,
        stdClass  $queryNameGenerator,
        Operation $operation = null,
        array     $context = [],
    ): void {}
}
