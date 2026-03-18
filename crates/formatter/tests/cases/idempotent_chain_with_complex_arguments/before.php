<?php

$mock->expects($this->once())->method('serialize')->with([], 'json', ['headers' => [], 'delivery_mode' => 2]);

$result = $service
    ->configure($options)
    ->process($data)
    ->transform(match ($type) {
        'json' => new JsonTransformer(),
        'xml' => new XmlTransformer(),
    });

$result = $factory->create()->configure('json', true)->execute('default');

$table->addColumn('total', Types::STRING, ['length' => 3, 'precision' => 10])->setNotnull(false);

$query
    ->select('*')
    ->from('users')
    ->where(function ($q) {
        $q->where('active', true);
    });

$builder
    ->setHeader('Content-Type', 'application/json')
    ->setBody(json_encode(['key' => 'value', 'nested' => ['a' => 1]]))
    ->send();

TestCase::create()
    ->expects($this->once())
    ->method('handle')
    ->willReturn(['status' => 'ok', 'code' => 200]);

$repository->findBy(function ($item) { return $item->isActive(); })->sortBy('name');

class IdempotencyTest
{
    public function testChainWithExpandedArray()
    {
        $builder->select('*')->from('users')->whereFullText('body', '+Hello -World', [
            'mode' => 'boolean',
            'expanded' => true,
        ]);

        static::createClient()->request('GET', '/books/1', options: ['headers' => ['accept' => 'application/ld+json']]);

        $this->artisan('test:select')->expectsChoice('What is your name?', 'jane', [
            'john' => 'John',
            'jane' => 'Jane',
        ])->expectsOutput('Your name is jane.');
    }
}
