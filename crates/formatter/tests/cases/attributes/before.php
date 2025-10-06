<?php

#[Deprecated]
const FOO = 'foo';

#[ATTRIBUTE]
function aaa(
  int $bbb,
  // this comment causes attribute to disappear
  int $ccc
) {
  var_dump('test');
}



#[Route('route/path', name: 'very_very_very_very_very_very_long_route_name', methods: ['GET']
)
]
        class Foo {}
        
        #[
        Route('route/path', name: 'very_very_very_very_very_very_very_very_very_very_very_very_very_very_very_very_long_route_name', methods: ['GET']
        )
        ]
                class Foo {}
        
        
        
        #[
        ORM\Entity
        (
        repositoryClass: InvoiceRepository::class
        )
        ]
        #[
        ORM\Table
        (
        name: 'invoices'
        )
        ]
        #[
        ORM\Index
        (
        name: 'idx_invoice_status', columns: ['status']
        )
        ]
        #[
        ORM\Index
        (
        name: 'idx_invoice_client', columns: ['client_id']
        )
        ]
        #[
        ORM\Index
        (
        name: 'idx_invoice_issue_date', columns: ['issue_date']
        )
        ]
        #[
        ORM\Index
        (
        name: 'idx_invoice_reference', columns: ['reference']
        )
        ]
        class Invoice extends Entity
        {
            public const int DEFAULT_PAYMENT_TERMS_DAYS = 15;
        
            #[
            ORM\Column
            (
            type: Types::STRING, length: 32, unique: true
            )
            ]
            public string $reference;
        
            #[
            ORM\ManyToOne
            (
            targetEntity: Client::class
            )
            ]
            #[
            ORM\JoinColumn
            (
            name: 'client_id', referencedColumnName: 'id', nullable: false
            )
            ]
            public Client $client;
        
            #[
            ORM\Column
            (
            type: Types::DATE_IMMUTABLE
            )
            ]
            public DateTimeImmutable $issueDate;
        
            #[
            ORM\Column
            (
            type: Types::INTEGER, options: ['default' => self::DEFAULT_PAYMENT_TERMS_DAYS]
            )
            ]
            public int $paymentTermsDays = self::DEFAULT_PAYMENT_TERMS_DAYS;
        
            #[
            ORM\Column
            (
            type: Types::TEXT, nullable: true
            )
            ]
            public null|string $note = null;
        
            #[
            ORM\OneToOne
            (
            targetEntity: Payment::class, cascade: ['persist', 'remove']
            )
            ]
            #[
            ORM\JoinColumn
            (
            name: 'payment_id', referencedColumnName: 'id', nullable: true
            )
            ]
            public null|Payment $payment = null;
        
            #[
            ORM\ManyToOne
            (
            targetEntity: BankAccount::class
            )
            ]
            #[
            ORM\JoinColumn
            (
            name: 'payment_account_id', referencedColumnName: 'id', nullable: false
            )
            ]
            public BankAccount $paymentAccount;
        
            #[
            ORM\Column
            (
            type: Types::STRING,
             length: 20, enumType: InvoiceStatus::class, options: ['default' => InvoiceStatus::Draft]
                )
                ]
            public InvoiceStatus $status = InvoiceStatus::Draft;
            
            
            
            /**
             * @var Collection<int, InvoiceItem>
             */
            #[
             ORM\OneToMany(
                targetEntity: InvoiceItem::class,
            mappedBy: 'invoice',
                cascade: 
                ['persist', 'remove'],
                orphanRemoval: true,
            
                
                )
                ]
            public Collection $items;
        
            /**
             * @var Collection<int, InvoiceService>
             */
            #[
            ORM\OneToMany
            

            (
                targetEntity: InvoiceService::class
                ,
                mappedBy: 'invoice',
                cascade: ['persist', 'remove'],
                orphanRemoval: true,
            )
            ]
           
            public
            Collection $services
            ;
}