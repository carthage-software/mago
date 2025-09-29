<?php

declare(strict_types=1);

namespace App\Form\Expense;

use App\CQRS\Query\Document\GetDocumentsQuery;
use App\CQRS\QueryBusInterface;
use App\DataTransferObject\Expense\CreateExpense;
use App\Entity\Document\Document;
use App\Enum\Document\DocumentType;
use App\Enum\Expense\ExpenseCategory;
use Override;
use Symfony\Bridge\Doctrine\Form\Type\EntityType;
use Symfony\Component\Form\AbstractType;
use Symfony\Component\Form\Extension\Core\Type\CurrencyType;
use Symfony\Component\Form\Extension\Core\Type\DateType;
use Symfony\Component\Form\Extension\Core\Type\EnumType;
use Symfony\Component\Form\Extension\Core\Type\MoneyType;
use Symfony\Component\Form\Extension\Core\Type\TextareaType;
use Symfony\Component\Form\FormBuilderInterface;
use Symfony\Component\OptionsResolver\OptionsResolver;

final class CreateExpenseType extends AbstractType
{
    public function __construct(
        private readonly QueryBusInterface $queryBus,
    ) {}

    #[Override]
    public function buildForm(FormBuilderInterface $builder, array $options): void
    {
        $documents = $this->queryBus->ask(new GetDocumentsQuery(DocumentType::Receipt));

        $builder->add('date', DateType::class, [
            'required' => true,
        ])->add('amount', MoneyType::class, [
            'required' => true,
            'currency' => false,
        ])->add('currency', CurrencyType::class, [
            'required' => true,
        ])->add('category', EnumType::class, [
            'class' => ExpenseCategory::class,
            'choice_label' => fn(ExpenseCategory $category): string => $category->getLabel(),
            'required' => true,
        ])->add('description', TextareaType::class, [
            'required' => true,
            'empty_data' => '',
        ])->add('document', EntityType::class, [
            'class' => Document::class,
            'choices' => $documents,
            'choice_label' => fn(Document $document): string => $document->name,
            'placeholder' => 'No document',
        ]);
    }

    #[Override]
    public function configureOptions(OptionsResolver $resolver): void
    {
        $resolver->setDefaults([
            'data_class' => CreateExpense::class,
        ]);
    }
}
