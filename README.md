# i-hate-latex

Nom provisoire: __Cowtchoox__

Ok, je commence par faire ce doc pour savoir ce qu'il faut faire (au secours trop de librairies)



Candidats pour convertir le HTML en PDF:
- https://github.com/spipu/html2pdf: tourne sur un navigateur en PHP (un peu lourd), ne supporte pas toutes les fonctionnalités de HTML mais supporte bien les trucs de PDF
- https://github.com/parallax/jsPDF: tourne sur un navigateur en JS (un peu lourd aussi), mais j'ai beaucoup de doutes sur la conversion (je crois qu'il génère un image de la page)
- https://github.com/marcbachmann/node-html-pdf: nodejs, plus maintenu depuis longtemps, mais pas très grave vu ques les fichiers source envoyés seront générés par le programme (et donc on est sur qu'il ne va pas utiliser les dernières modifications de CSS)
- https://github.com/rust-headless-chrome/rust-headless-chrome: (besoin d'un chromium installé?)

Apparemment Firefox utilise Microsoft print to PDF, mais on va s'abaisser à utiliser un produit microsoft quand même...


Pour la syntaxe à utiliser:
- je vois bien du HTML, mais en plus d'avoir un `<head>` et `<body>`, on aurait genre `<header>` `<footer>`
- des balises supplémentaire comme `<pagebreak>` et `<maths>`
- un truc qui remplace une balise `<name args...>` par le contenu d'un autre fichier (j'ai deja essayé de faire un truc comme ca c'est pas cure et c'est pratique, même pour implementer les trucs qui seront là par défaut)
- pour les maths, on se démerde pour avoir les opérateurs usuels (idéalement un ou deux caractères par opérateur, par exemple , ou), des trucs de layout (`^`, `_`), et `{}` pour le parenthésage 
- garder tout le support des balises à l'intérieur des maths

Idées de syntaxe (juste des idées)
- `v/` pour la racine
- `*` pour `\times`
- `...` pour les dots
- `/` pour les fractions (en infixe)
- `%` pour la congruence
- `__` pour `\underset` (peut être en infixe?)
- `^^` pour `\overset`
- `!` pour barrer un truc
- `->`, `-->`, `=>`, etc, pour les flèches
- `inf` pour `\infty`
- `?E` et `?A` pour `\forall` et il existe
- `?U ?I ?C` pour union, inter, inclusion
- `€` pour l'appartenance
- `[|` et `|]` pour les intervalles entiers
- `~` pour équivalent

- `||` pour `\mathbb`
- `£`, `@` pour le calligraphié et le gothique
- `§` pour transformer en lettre grecque
- `¤` pour mettre en grand (grand sigma)

- `^_` pour `\overbar`
- `^{` pour `\overbrace`
- `_{` pour `\underbrace`
- `^>` pour les vecteurs

- `$` pour séparer les maths en plein de `<span>` et pouvoir aligner des trucs facilement grâce à CSS

Pour le reste, on utilisera des balises html (par exemple `<matrix></matrix>`) qui pourront être définies par l'utilisateur

