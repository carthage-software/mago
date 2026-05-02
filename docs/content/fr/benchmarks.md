+++
title = "Benchmarks"
description = "Comment Mago se compare aux autres outils PHP, et les compromis que nous avons faits pour y arriver."
nav_order = 20
nav_section = "Référence"
+++
# Benchmarks

Mago est conçu pour être la chaîne d'outils PHP la plus rapide. Chaque composant, du parseur à l'analyseur, a été pensé autour de cette contrainte.

Nous comparons Mago au reste de l'écosystème PHP de manière régulière. Les chiffres et l'historique se trouvent sur le tableau de bord dédié :

- [PHP Toolchain Benchmarks](https://carthage-software.github.io/php-toolchain-benchmarks/?project=psl&kind=Analyzers)
- [Dépôt source](https://github.com/carthage-software/php-toolchain-benchmarks)

Le tableau de bord exécute trois benchmarks sur plusieurs bases de code PHP réelles :

- Formateur : temps nécessaire pour vérifier le formatage d'une base de code entière.
- Linter : temps nécessaire pour linter une base de code entière.
- Analyseur : temps nécessaire pour effectuer une analyse statique complète depuis un cache froid.

## La promesse de performance

La vitesse n'est pas un objectif, c'est une garantie. Si un outil listé dans les benchmarks dépasse un jour Mago sur une comparaison équivalente, nous traitons cela comme un bug à haute priorité.

## À propos de la mémoire

L'utilisation mémoire varie selon la tâche. Pour l'analyse statique, Mago utilise généralement moins de mémoire que les alternatives (environ 3,5 fois moins que Psalm dans nos exécutions). Pour le linting et le formatage, Mago peut utiliser plus de mémoire qu'un outil PHP mono-thread.

C'est délibéré. Mago donne la priorité à votre temps plutôt qu'aux ressources machine.

Pour atteindre les vitesses que nous voulons, Mago utilise des allocateurs d'arène par thread. Plutôt que de demander de la mémoire au système d'exploitation pour chaque petit objet, il réserve de gros blocs à l'avance et alloue à l'intérieur à un coût quasi nul. Cela permet un parallélisme important, au prix d'une empreinte mémoire de pointe plus élevée sur certaines charges de travail. Nous estimons qu'échanger quelques centaines de mégaoctets de RAM contre plusieurs secondes (ou minutes) de temps développeur est un compromis raisonnable.
