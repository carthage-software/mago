<?php declare(strict_types=1);

// Test @inheritDoc inheritance from parent class

class BaseProcessor
{
    /**
     * @param non-empty-string $data String data
     * @return array<int, string> Processed as array
     */
    public function process(string $data): array
    {
        return ['result'];
    }
}

class DerivedProcessor extends BaseProcessor
{
    /** @inheritDoc */
    public function process(string $data): array
    {
        // Should inherit from BaseProcessor:
        // @param string $data
        // @return array<int, string>
        $result = parent::process($data);
        foreach ($result as $index => $value) {
            // $index should be int, $value should be string
            echo $index . ': ' . $value;
        }
        return $result;
    }
}

$processor = new DerivedProcessor();
$processor->process('test data');
