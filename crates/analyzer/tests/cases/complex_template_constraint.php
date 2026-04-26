<?php

class UploadedFile
{
    public function __construct(
        public string $name,
        public int $size,
        public string $type,
        public string $tmp_name,
        public bool $error,
    ) {}
}

/**
 * @template T of array{name?: string, age?: string, certificates?: UploadedFile|list<UploadedFile>|array<string,UploadedFile>}
 */
class FormData
{
    /**
     * @param T $data
     */
    public function __construct(
        public array $data,
    ) {}

    /**
     * @return T
     */
    public function getData(): array
    {
        return $this->data;
    }
}

/**
 * @param FormData<array{name?: string, age?: string, certificates?: UploadedFile|list<UploadedFile>|array<string,UploadedFile>}> $formData
 *
 * @return array<UploadedFile>
 */
function get_certificates_1(FormData $formData): array
{
    $certificates = $formData->getData()['certificates'] ?? [];
    if ($certificates instanceof UploadedFile) {
        return [$certificates];
    }

    $result = [];
    foreach ($certificates as $certificate) {
        $result[] = $certificate;
    }

    return $result;
}

// FormData's `T` is invariant because the public `$data` property is read/write.
// Each `new FormData([...])` infers a more specific shape (required keys, literal
// strings) than the function parameter. Strict invariance correctly rejects the
// flow since the callee could otherwise overwrite `$data` with a value that
// violates the original narrower shape. PHPStan reports the same.
/** @mago-expect analysis:invalid-argument */
$certificates = get_certificates_1(new FormData([
    'name' => 'John Doe',
    'age' => '30',
    'certificates' => [
        new UploadedFile('certificate1.pdf', 2048, 'application/pdf', '/tmp/certificate1.pdf', false),
        new UploadedFile('certificate2.pdf', 1024, 'application/pdf', '/tmp/certificate2.pdf', false),
    ],
]));

/** @mago-expect analysis:invalid-argument */
$certificates2 = get_certificates_1(new FormData([
    'name' => 'Jane Smith',
    'age' => '25',
    'certificates' => new UploadedFile('certificate3.pdf', 3072, 'application/pdf', '/tmp/certificate3.pdf', false),
]));

/** @mago-expect analysis:invalid-argument */
$certificates3 = get_certificates_1(new FormData([
    'name' => 'Alice Johnson',
    'age' => '28',
    'certificates' => [
        'cert1' => new UploadedFile('certificate4.pdf', 4096, 'application/pdf', '/tmp/certificate4.pdf', false),
        'cert2' => new UploadedFile('certificate5.pdf', 5120, 'application/pdf', '/tmp/certificate5.pdf', false),
    ],
]));

$all_certificates = [
    ...$certificates,
    ...$certificates2,
    ...$certificates3,
];

foreach ($all_certificates as $certificate) {
    echo
        "Certificate Name: {$certificate->name}, Size: {$certificate->size} bytes, Type: {$certificate->type}, Temp Name: {$certificate->tmp_name}\n"
    ;
}
