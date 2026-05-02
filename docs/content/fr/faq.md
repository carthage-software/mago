+++
title = "FAQ"
description = "Questions fréquentes sur Mago, le projet, et ce qui y a sa place ou non."
nav_order = 10
nav_section = "Référence"
+++
# FAQ

## Pourquoi le nom « Mago » ?

Le projet s'appelait à l'origine « fennec », d'après le fennec, renard du désert d'Afrique du Nord. Un conflit de nom avec un autre outil a forcé un changement.

Nous avons choisi « Mago » pour rester proches de nos racines chez Carthage Software. Mago de Carthage était un écrivain carthaginois antique connu comme le « Père de l'Agriculture ». Tout comme il cultivait la terre, l'outil vise à aider les développeurs à cultiver leurs bases de code.

Le nom a un double sens utile. En espagnol et en italien, « mago » signifie « magicien » ou « sorcier ». Le logo capture les deux : un fennec coiffé d'un chapeau et d'une robe de sorcier, avec l'ancien symbole carthaginois de Tanit sur ses vêtements.

## Comment prononce-t-on Mago ?

`/ˈmɑːɡoʊ/`, « ma-go ». Deux syllabes : « ma » comme dans « maman », « go » comme dans « go ».

## Mago implémentera-t-il un LSP ?

Oui. L'implémentation du Language Server Protocol est prévue pour la `2.0.0`. Elle était initialement planifiée pour la `1.0.0`, mais a été décalée afin que le LSP arrive complet plutôt que comme une première version minimale.

Pour le détail, voir le billet de blog [Why Mago 1.0.0 Won't Ship With an LSP](https://carthage.software/en/blog/article/Why-Mago-1-0-0-Won-t-Ship-With-an-LSP).

## Mago proposera-t-il des extensions d'éditeur (VS Code, etc.) ?

Non. Le projet se concentrera sur l'implémentation du standard LSP et ne maintiendra pas d'extensions spécifiques à un éditeur. Les éditeurs qui prennent en charge l'intégration LSP (Helix, Neovim via lspconfig, VS Code avec un client générique) fonctionneront avec Mago. Nous encourageons la communauté à construire des wrappers spécifiques à un éditeur et serons heureux de mettre en avant les plus appréciés sur le site.

## Mago prendra-t-il en charge des plugins d'analyseur ?

Oui, mais pas avant la `1.0.0`. Le plan est que les plugins soient écrits en Rust, compilés en WASM, et chargés par Mago à l'exécution. Ce travail aura lieu après la sortie de la `1.0.0`.

## Quels autres outils PHP Mago prévoit-il de remplacer ?

La vision à plus long terme est que Mago soit un utilitaire complet de qualité et de développement pour PHP. Le formateur, le linter et l'analyseur sont la priorité pour la `1.0.0`. Au-delà, les outils prévus incluent :

- Un gestionnaire de versions PHP.
- Un installateur d'extensions PHP.
- Un assistant de migration pour mettre à niveau les versions de PHP, les frameworks ou les bibliothèques.

## Mago implémentera-t-il une alternative à Composer ?

Non. Composer est un outil fantastique, et l'essentiel de son travail est lié aux I/O. Une réécriture en Rust n'apporterait pas grand-chose en vitesse, fragmenterait l'écosystème et rendrait très difficile la prise en charge de l'architecture de plugins de Composer basée sur PHP.

## Mago implémentera-t-il un runtime PHP ?

Non. Le runtime PHP est énorme. Même de très gros efforts (HHVM de Facebook, KPHP de VK) ont eu du mal à atteindre la pleine parité avec le moteur Zend. Un projet plus petit ne peut pas faire mieux, et le résultat ne ferait que fragmenter la communauté. Mago se concentre sur l'outillage, pas sur les runtimes.
