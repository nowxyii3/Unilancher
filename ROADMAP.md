# 🚀 UniLauncher — Feuille de Route & Fonctionnalités Réalisées

Ce document recense l'ensemble des fonctionnalités et améliorations d'interface (UI/UX) conçues et intégrées pour faire d'**UniLauncher** le launcher unifié ultime pour PC, combinant la rapidité de Rust, l'élégance de GOG Galaxy / Steam et la flexibilité de Playnite.

---

## 📌 Sommaire
1. [Interface Utilisateur & Expérience Visuelle (UI/UX)](#-1-interface-utilisateur--expérience-visuelle-uiux)
2. [Métadonnées & Enrichissement Automatique](#-2-métadonnées--enrichissement-automatique)
3. [Outils Gaming & Fonctionnalités Avancées (Power-User)](#-3-outils-gaming--fonctionnalités-avancées-power-user)
4. [Intégrations & Multi-Stores](#-4-intégrations--multi-stores)
5. [Matrice d'Implémentation Complète](#-5-matrice-dimplémentation-complète)

---

## 🎨 1. Interface Utilisateur & Expérience Visuelle (UI/UX)

### 1.1. Ambiance Lumineuse Dynamique (Ambient Glow)
* **Description :** Lors du survol d'un jeu, projection d'un halo lumineux doux (« bloom / backlight ») en arrière-plan calculé à partir de la couleur emblématique de la plateforme (Steam, Epic, GOG, EA, Xbox...).
* **État :** ✅ Implémenté dans le composant `game_card`.

### 1.2. Curseur de Zoom de la Grille (Taille des Jaquettes)
* **Description :** Curseur ergonomique dans la barre d'outils supérieure permettant d'ajuster l'échelle des cartes de 70% à 140% instantanément avec adaptation dynamique des polices et espacements.
* **État :** ✅ Implémenté (`card_scale` stocké dans la configuration).

### 1.3. Hero Banner Dynamique
* **Description :** Grand bandeau moderne en haut de la bibliothèque présentant le dernier jeu lancé, son temps de jeu cumulé, sa plateforme et un bouton « Reprendre la session ».
* **État :** ✅ Implémenté (`hero_banner`).

### 1.4. Mode « Big Picture » / Console (Navigation Manette & Clavier)
* **Description :** Vue plein écran épurée optimisée pour écran de salon / TV avec affichage géant de la jaquette (260x370), carrousel horizontal inférieur, et pilotage complet via raccourcis clavier et manette (Touches Fléchées, Entrée, Espace, F pour favoris, F10 / Échap pour basculer).
* **État :** ✅ Implémenté (`big_picture_ui`, raccourci direct `F10`).

### 1.5. Thèmes & Personnalisation Poussée
* **Description :** Sélecteur de thèmes en temps réel proposant 4 ambiances complètes :
  * **Obsidian Zinc :** Tons sombres neutres et modernes.
  * **OLED Noir Pur :** Fond noir absolu (`#000000`) pour dalles OLED.
  * **Cyberpunk Néon :** Violet profond et contrastes cyberpunk.
  * **Midnight Blue :** Bleu nuit profond façon interface PlayStation.
* **État :** ✅ Implémenté (`ThemeColors`, `get_theme` et sélecteur dans l'onglet Paramètres).

---

## 🌐 2. Métadonnées & Enrichissement Automatique

### 2.1. Téléchargement Automatique des Jaquettes HD
* **Description :** Pour les jeux manuels ou sans jaquette locale, recherche et téléchargement automatique en arrière-plan via les CDN Steam et l'API Steam Store publique, avec mise en cache locale permanente dans `.unilauncher/covers/`.
* **État :** ✅ Implémenté (`download_cover_auto` dans `scan.rs`).

### 2.2. Raccourcis Communautaires en 1 Clic
* **Description :** Boutons d'accès directs sur le tiroir latéral de chaque jeu :
  * 🌐 **PCGamingWiki :** Corrections de bugs, patchs ultrawide, FOV et astuces graphiques.
  * 🧩 **NexusMods :** Page de mods du jeu.
  * ⏳ **HowLongToBeat :** Estimation de la durée de vie du jeu.
  * 🐧 **ProtonDB :** Compatibilité Linux et Steam Deck.
* **État :** ✅ Implémenté dans `details_drawer_ui`.

---

## ⚡ 3. Outils Gaming & Fonctionnalités Avancées (Power-User)

### 3.1. Sauvegarde et Sécurisation des Données de Jeu (Save Backup)
* **Description :** Détection automatique des répertoires de sauvegardes PC (`%USERPROFILE%/Saved Games`, `%APPDATA%`, `Documents/My Games`) et génération en un clic d'une archive `.zip` datée dans `.unilauncher/backups/`.
* **État :** ✅ Implémenté (`find_save_folder` & `create_save_backup`).

### 3.2. Analyseur d'Espace Disque & Tri par Taille
* **Description :** Calcul récursif de l'espace occupé sur le stockage par chaque dossier d'installation, affichage formaté (Mo / Go) sur les fiches et dans la vue liste, et nouveau mode de tri **Taille sur le disque**.
* **État :** ✅ Implémenté (`SortMode::SizeOnDisk` & `calculate_folder_size`).

### 3.3. Gaming Analytics & Tableau de Bord Statistiques
* **Description :** Onglet dédié « Statistiques » avec métriques globales (nombre de titres, temps total de jeu, volume disque total, plateformes connectées), barres de répartition par boutique, Top 5 des jeux les plus joués et Top 5 des jeux les plus volumineux.
* **État :** ✅ Implémenté (`Tab::Stats` & `stats_ui`).

### 3.4. Mode « Performance Boost » au Lancement
* **Description :** Option dans les réglages basculant automatiquement le mode d'alimentation Windows en « Performances élevées » (GUID `8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c`) dès qu'un jeu démarre, et rétablissant le profil équilibré dès la fin de session de jeu.
* **État :** ✅ Implémenté (`performance_mode` dans `launch_game` et `Launcher::update`).

---

## 🔌 4. Intégrations & Multi-Stores

| Plateforme | Méthode de détection |
| :--- | :--- |
| **Steam** | Détection automatique des bibliothèques via `libraryfolders.vdf` et manifestes `.acf`. |
| **Epic Games** | Détection des manifestes d'installation `.item` dans `ProgramData\Epic\EpicGamesLauncher`. |
| **GOG Galaxy** | Analyse de la base SQLite `galaxy-2.0.db` et des registres Windows. |
| **Ubisoft Connect** | Analyse des clés de registre `HKLM\SOFTWARE\WOW6432Node\Ubisoft\Launcher\Installs`. |
| **EA App / Origin** | Analyse des fichiers de licence et manifestes EA dans `ProgramData\EA Desktop`. |
| **Xbox / Microsoft Store** | Détection automatique dans les répertoires `XboxGames` et `ModifiableWindowsApps`. |
| **Dossiers Personnalisés** | Scanner récursif d'exécutables (`.exe`) et d'affiches d'images dans les dossiers choisis. |
| **Discord Rich Presence 2.0** | Statut en jeu en direct avec thread asynchrone non-bloquant, reconnexion automatique toutes les 3s et Client ID Discord configurable. |

---

## 🎯 5. Matrice d'Implémentation Complète

| Fonctionnalité | Catégorie | Statut | Résultat |
| :--- | :---: | :---: | :--- |
| **Ambiance Lumineuse (Ambient Glow)** | UI/UX | ✅ Opérationnel | Halo coloré dynamique sur les cartes de jeux au survol. |
| **Curseur de Zoom Grille (70% - 140%)** | UI/UX | ✅ Opérationnel | Redimensionnement direct sans perte de netteté. |
| **Hero Banner Dynamique** | UI/UX | ✅ Opérationnel | Bannière interactive en haut de la bibliothèque. |
| **Mode Big Picture / Console (F10)** | UI/UX | ✅ Opérationnel | Vue TV / manette grand écran avec carrousel fluide. |
| **4 Thèmes (Obsidian, OLED, Cyber, Blue)** | UI/UX | ✅ Opérationnel | Changement de palette instantané et persistant. |
| **Téléchargement Auto des Jaquettes HD** | Métadonnées | ✅ Opérationnel | Remplacement automatique des jaquettes par défaut via Steam CDN. |
| **4 Raccourcis Communautaires (PCGW, Nexus, HLTB, Proton)** | Métadonnées | ✅ Opérationnel | Liens directs depuis la fiche détaillée du jeu. |
| **Sauvegarde des Saves (Backup ZIP)** | Power-User | ✅ Opérationnel | Export zippé horodaté des sauvegardes locales. |
| **Taille Disque & Tri par Espace Occupé** | Power-User | ✅ Opérationnel | Calcul de la taille et filtre par jeux les plus lourds. |
| **Dashboard Analytics & Statistiques** | Power-User | ✅ Opérationnel | Graphiques et podiums des jeux les plus joués et les plus volumineux. |
| **Mode Performance Boost Windows** | Power-User | ✅ Opérationnel | Optimisation du profil énergétique pendant les sessions de jeu. |
| **Discord Rich Presence 2.0 Asynchrone** | Réseau | ✅ Opérationnel | Affichage fiable du statut Discord sans gel ni blocage d'UI. |
