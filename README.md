# i-hate-latex

Nom provisoire: __Cowtchoox__

## Organisation du code
Rust: le programme principal
- `main`: lire l'entrée, lire les fichiers
- `parser`: transformer le fichier COW en une struct
- `writer`: transformer la struct en HTML
- `browser`: envoyer le fichier au navigateur

JS: tourne dans le navigateur, sert à découper les pages comme il faut et à insérer les headers (le navigateur ne fait pas les headers et coupe n'importe comment)
- J'ai rien fait pout l'instant

On va utiliser la crate headless_chrome pour la conversion en PDF

## Syntaxe
- je vois bien du HTML, mais en plus d'avoir un `<head>` et `<body>`, on aurait genre `<header>` `<footer>`
- des balises supplémentaire comme `<pagebreak>` et `<maths>`, et un alias `$` pour `<maths>`
- un truc qui remplace une balise (avec majuscule, come React) `<Name args...>` par le contenu d'un autre fichier (j'ai deja essayé de faire un truc comme ca c'est pas dur et c'est pratique, même pour implementer les trucs qui seront là par défaut)
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
- `?E` (`?E!`) et `?A` pour `\forall` et il existe (un unique)
- `?u ?i ?c` pour union, inter, inclusion
- `?U` `?I` pour big_cup et big_cap
- `€` pour l'appartenance
- `[|` et `|]` pour les intervalles entiers
- `~` pour équivalent
- `~=` pour arrondis...
- `|,` `,|` et `|'` `'|` pour \floor et \cel
- `!]` pour indiquer que le ] doit matcher avec un autre dans le mauvais sens (intervalles `]a; b[` s'écrivent `!]a; b[!`)

- `||` pour `\mathbb`
- `£`, `@` pour le calligraphié et le gothique
- `§` pour transformer en lettre grecque
- `¤` pour mettre en grand (grand sigma)

- `^_` pour `\overbar`
- `^{` pour `\overbrace`
- `_{` pour `\underbrace`
- `^>` pour les vecteurs

- `&` pour séparer les maths en plein de `<span>` et pouvoir aligner des trucs facilement grâce à CSS

- `;;` pour le retour à la ligne 
- `//` pour un commentaire

Pour le reste, on utilisera des balises html (par exemple `<matrix></matrix>`) qui pourront être définies par l'utilisateur


### Autres candidats pour convertir le HTML en PDF:
- https://github.com/spipu/html2pdf: tourne sur un navigateur en PHP (un peu lourd), ne supporte pas toutes les fonctionnalités de HTML mais supporte bien les trucs de PDF
- https://github.com/parallax/jsPDF: tourne sur un navigateur en JS (un peu lourd aussi), mais j'ai beaucoup de doutes sur la conversion (je crois qu'il génère un image de la page)
- https://github.com/marcbachmann/node-html-pdf: nodejs, plus maintenu depuis longtemps, mais pas très grave vu ques les fichiers source envoyés seront générés par le programme (et donc on est sur qu'il ne va pas utiliser les dernières modifications de CSS)
- https://github.com/rust-headless-chrome/rust-headless-chrome: (besoin d'un chromium installé?)
