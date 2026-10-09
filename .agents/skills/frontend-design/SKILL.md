---
name: frontend-design
description: Protocole d'ingénierie et de design frontend pour générer des interfaces à forte valeur graphique, hautement différenciées et exemptes de patterns génériques d'IA.
version: 1.0.0
---

# Frontend Design Skill Specification

## 1. Objectif & Mandats

Ce skill force l'agent à abandonner les conventions visuelles génériques (« AI slop » : boutons indigo/violet par défaut, grilles ternaires de cartes identiques, police Inter par défaut, bordures floues à faible contraste) au profit de choix visuels assumés, rigoureux et prêts pour la production.

### Principes directeurs :

1. **Thèse esthétique déclarée** : Chaque composant ou page doit adopter une direction artistique précise et documentée (ex. _Industrial Utilitarian_, _Swiss Minimal_, _Terminal High-Density_, _Editorial Brutalism_).
2. **Ancre de différenciation** : Au moins un élément de mise en page, de typographie ou de hiérarchie visuelle doit conférer une signature immédiate à l'interface.
3. **Qualité logicielle** : HTML5 sémantique strict, gestion des états interactifs (`:hover`, `:focus-visible`, `:active`, `:disabled`), support responsive natif et respect des règles d'accessibilité WCAG 2.1 AA.
4. **Retenue structurelle** : Aucun ornement gratuit, aucun mouvement sans finalité ergonomique.

---

## 2. Matrice d'évaluation DFII (Design Feasibility & Impact Index)

Avant d'écrire le code, évaluer l'approche selon la formule :

$$\text{DFII} = (\text{Impact Esthétique} + \text{Pertinence Contexte} + \text{Faisabilité} + \text{Performance}) - \text{Risque de Cohérence}$$

Chaque facteur est noté de 1 à 4, le risque de 1 à 3 :

- **12 à 15** : Direction validée, procéder à l'implémentation complète.
- **8 à 11** : Direction solide, cadrer strictement la portée et les variantes.
- **≤ 7** : Complexité superflue ou concept confus. Épurer la mise en page et revenir à des tokens fondamentaux.

---

## 3. Règles d'implémentation & Anti-Patterns

### Typographie

- **Interdit** : Charger Inter, Roboto ou System sans configuration éditoriale ; appliquer des dégradés de couleur sur un seul mot dans les titres ; multiplier les graisses sans hiérarchie d'échelle.
- **Exigé** : Définir un contraste marqué entre une typographie de titrage expressive et une typographie de labeur neutre et lisible. Utiliser des échelles de taille cohérentes (ex. ratio `1.25` ou `1.333`).

### Couleur & Tokens

- **Interdit** : Boutons primaires en violet/bleu générique (`#6366f1` / `#8b5cf6`) sans justification thématique ; palettes sans contraste ou dégradés d'arrière-plan omniprésents.
- **Exigé** : Établir une palette basée sur des tokens sémantiques (CSS variables ou config Tailwind étendue) :
  - `surface-base`, `surface-raised`, `surface-overlay`
  - `text-primary`, `text-secondary`, `text-muted`
  - Une seule couleur d'accentuation dominante avec ses nuances d'interaction
  - Contraste suffisant entre le texte et les surfaces d'arrière-plan

### Layout & Rythme

- **Interdit** : Grilles répétitives composées de 3 cartes avec une icône entourée d'un cercle coloré en haut à gauche ; espacements arbitraires (`gap-7`, `p-5` aléatoires).
- **Exigé** : Rythme vertical basé sur un facteur 4 ou 8 (`gap-4`, `gap-8`, `p-6`). Utiliser l'asymétrie délibérée, les zones de respiration et une gestion stricte de la densité selon le cas d'usage (haute densité pour les dashboards, aérée pour l'éditorial).

### Motion & Micro-interactions

- **Interdit** : Éléments qui flottent en permanence, effets de rebond (`bounce`) non sollicités, animations d'entrée retardant l'affichage des informations critiques.
- **Exigé** : Transitions courtes (150ms à 250ms, `cubic-bezier(0.16, 1, 0.3, 1)`), transitions d'état strictement fonctionnelles (changement de statut, validation de formulaire, ouverture de dialogue).

---

## 4. Format de sortie attendu

Toute génération frontend produite via ce skill doit suivre cette structure :

1. **Intention & Thèse** : 2 lignes expliquant la direction visuelle et le motif des choix typographiques/colorimétriques.
2. **Tokens / Styles de base** : Variables CSS ou classes utilitaires clés définissant la palette et l'échelle.
3. **Composant(s)** : Code propre, typé (si TypeScript/React) ou HTML/CSS structuré, immédiatement exécutable sans dépendance superflue.
