-- Almanach Seed Data: Ontario Intro to Computer Science, Grade 10 (ICD20)
-- Semester 1 / Semestre 1
-- Insert this after SCHEMA_COURSES tables exist.

-- Course
INSERT INTO courses (id, code, title, title_en, description, grade, language, credit_hours, created_at)
VALUES (
    'course-icd20-2026',
    'ICD20',
    'Introduction à l''informatique, 10e année',
    'Introduction to Computer Science, Grade 10',
    'Ontario Grade 10 computer science covering problem-solving, programming fundamentals, data representation, software development, and digital citizenship.',
    '10',
    'fr',
    1,
    CURRENT_TIMESTAMP
);

-- Unit 1: Problem-Solving & Computational Thinking / Résolution de problèmes et pensée computationnelle (30 hours)
INSERT INTO modules (id, course_id, title, title_en, description, order_index, estimated_hours, created_at)
VALUES (
    'mod-icd20-u1',
    'course-icd20-2026',
    'Unité 1 : Résolution de problèmes et pensée computationnelle',
    'Unit 1: Problem-Solving & Computational Thinking',
    'Decomposition, pattern recognition, abstraction, algorithm design, and flowcharts.',
    1,
    30,
    CURRENT_TIMESTAMP
);

INSERT INTO lessons (id, module_id, title, title_en, description, topics, objectives, estimated_minutes, system_prompt, keywords, order_index, created_at) VALUES
('les-icd20-1-1', 'mod-icd20-u1', '1.1 — Qu''est-ce que l''informatique ?', '1.1 — What is Computer Science?', 'History, impact, and career paths in computing.', '["Informatique", "Histoire", "Carrières", "Technologie"]', '["Décrire l''évolution de l''informatique", "Identifier des carrières en technologie", "Évaluer l''impact social du numérique"]', 120, 'Tu es un mentor en informatique bienveillant. Explique les concepts en français simple. Utilise des anecdotes historiques et des exemples concrets.', '["informatique", "histoire", "carrières", "impact social"]', 1, CURRENT_TIMESTAMP),
('les-icd20-1-2', 'mod-icd20-u1', '1.2 — Décomposition d''un problème', '1.2 — Decomposition', 'Breaking complex problems into smaller, manageable parts.', '["Décomposition", "Sous-problèmes", "Complexité"]', '["Décomposer un problème complexe en parties simples", "Identifier les dépendances entre sous-problèmes", "Évaluer la difficulté relative de chaque partie"]', 120, 'Tu es un mentor en informatique. Utilise des analogies (recette de cuisine, assemblage de meubles). Encourage l''élève à décrire chaque étape.', '["décomposition", "sous-problèmes", "complexité", "analogies"]', 2, CURRENT_TIMESTAMP),
('les-icd20-1-3', 'mod-icd20-u1', '1.3 — Reconnaissance de motifs', '1.3 — Pattern Recognition', 'Identifying similarities and trends across problems.', '["Motifs", "Patterns", "Généralisation", "Abstraction"]', '["Identifier des motifs communs dans des problèmes variés", "Généraliser une solution à une classe de problèmes", "Utiliser l''abstraction pour simplifier"]', 120, 'Tu es un mentor en informatique. Montre des exemples visuels (formes géométriques, séquences numériques). Fais deviner la règle avant de la révéler.', '["motifs", "patterns", "généralisation", "abstraction"]', 3, CURRENT_TIMESTAMP),
('les-icd20-1-4', 'mod-icd20-u1', '1.4 — Conception d''algorithmes', '1.4 — Algorithm Design', 'Step-by-step instructions, pseudocode, and flowcharts.', '["Algorithme", "Pseudocode", "Organigramme", "Instructions"]', '["Écrire un algorithme en pseudocode", "Représenter un algorithme avec un organigramme", "Vérifier la correction d''un algorithme"]', 180, 'Tu es un mentor en informatique. Fais pratiquer avec des problèmes du quotidien (faire un sandwich, se rendre à l''école). Insiste sur la précision.', '["algorithme", "pseudocode", "organigramme", "précision"]', 4, CURRENT_TIMESTAMP);

-- Unit 2: Programming Fundamentals / Fondements de la programmation (35 hours)
INSERT INTO modules (id, course_id, title, title_en, description, order_index, estimated_hours, created_at)
VALUES (
    'mod-icd20-u2',
    'course-icd20-2026',
    'Unité 2 : Fondements de la programmation',
    'Unit 2: Programming Fundamentals',
    'Variables, data types, control structures, functions, and simple input/output in Python.',
    2,
    35,
    CURRENT_TIMESTAMP
);

INSERT INTO lessons (id, module_id, title, title_en, description, topics, objectives, estimated_minutes, system_prompt, keywords, order_index, created_at) VALUES
('les-icd20-2-1', 'mod-icd20-u2', '2.1 — Variables et types de données', '2.1 — Variables & Data Types', 'Strings, integers, floats, booleans, and type conversion.', '["Variables", "Types de données", "Chaînes", "Entiers", "Flottants", "Booléens"]', '["Déclarer et utiliser des variables", "Distinguer les types de données de base", "Effectuer des conversions de type"]', 120, 'Tu es un tuteur de programmation patient. Explique les variables comme des boîtes étiquetées. Utilise Python. Fais beaucoup d''exercices courts.', '["variables", "types", "python", "conversion", "déclaration"]', 1, CURRENT_TIMESTAMP),
('les-icd20-2-2', 'mod-icd20-u2', '2.2 — Opérations et expressions', '2.2 — Operations & Expressions', 'Arithmetic, comparison, and logical operators.', '["Opérateurs", "Expressions", "Arithmétique", "Comparaison", "Logique"]', '["Utiliser les opérateurs arithmétiques", "Évaluer des expressions booléennes", "Combiner des conditions avec AND/OR"]', 120, 'Tu es un tuteur de programmation. Fais calculer l''élève à la main avant de coder. Montre les pièges classiques (division entière, comparaison flottante).', '["opérateurs", "expressions", "arithmétique", "logique", "comparaison"]', 2, CURRENT_TIMESTAMP),
('les-icd20-2-3', 'mod-icd20-u2', '2.3 — Structures conditionnelles', '2.3 — Conditional Statements', 'if, elif, else, nested conditions.', '["Conditions", "if", "elif", "else", "Imbrication"]', '["Écrire des structures conditionnelles simples", "Utiliser elif pour plusieurs cas", "Gérer des conditions imbriquées"]', 180, 'Tu es un tuteur de programmation. Utilise des scénarios réels (notes, âge, prix). Fais tracer le flux d''exécution sur papier.', '["conditions", "if", "elif", "else", "imbrication", "flux"]', 3, CURRENT_TIMESTAMP),
('les-icd20-2-4', 'mod-icd20-u2', '2.4 — Boucles — while', '2.4 — Loops — while', 'Counter-controlled and sentinel-controlled while loops.', '["Boucles", "while", "Compteur", "Sentinelle"]', '["Écrire une boucle while avec compteur", "Utiliser une sentinelle pour arrêter", "Éviter les boucles infinies"]', 120, 'Tu es un tuteur de programmation. Insiste sur la condition d''arrêt. Fais prédire le nombre d''itérations avant d''exécuter.', '["boucles", "while", "compteur", "sentinelle", "infini"]', 4, CURRENT_TIMESTAMP),
('les-icd20-2-5', 'mod-icd20-u2', '2.5 — Boucles — for et range', '2.5 — Loops — for & range', 'Iterating over sequences and using range().', '["for", "range", "Itération", "Séquences"]', '["Utiliser for avec range()", "Itérer sur une liste de chaînes", "Choisir entre for et while"]', 120, 'Tu es un tuteur de programmation. Montre les deux façons de compter (0-based vs 1-based). Fais des exercices de parcours de listes.', '["for", "range", "itération", "séquences", "parcours"]', 5, CURRENT_TIMESTAMP),
('les-icd20-2-6', 'mod-icd20-u2', '2.6 — Listes et indices', '2.6 — Lists & Indexing', 'Creating, accessing, modifying, and slicing lists.', '["Listes", "Indices", "Slicing", "Modification"]', '["Créer et accéder à une liste", "Modifier des éléments par indice", "Utiliser le slicing pour extraire des sous-listes"]', 180, 'Tu es un tuteur de programmation. Utilise des listes concrètes (noms d''élèves, notes, températures). Montre les erreurs d''indice courantes.', '["listes", "indices", "slicing", "modification", "erreurs"]', 6, CURRENT_TIMESTAMP),
('les-icd20-2-7', 'mod-icd20-u2', '2.7 — Fonctions — Définition et appel', '2.7 — Functions — Definition & Call', 'Defining functions, parameters, return values, and scope.', '["Fonctions", "Paramètres", "Retour", "Portée"]', '["Définir une fonction avec paramètres", "Retourner une valeur", "Distinguer portée locale et globale"]', 180, 'Tu es un tuteur de programmation. Utilise l''analogie de la machine à café (entrées, processus, sortie). Fais refactorer du code dupliqué.', '["fonctions", "paramètres", "retour", "portée", "refactorisation"]', 7, CURRENT_TIMESTAMP),
('les-icd20-2-8', 'mod-icd20-u2', '2.8 — Fonctions — Bibliothèques standard', '2.8 — Functions — Standard Libraries', 'Importing and using math, random, and datetime modules.', '["Bibliothèques", "import", "math", "random", "datetime"]', '["Importer et utiliser un module standard", "Générer des nombres aléatoires", "Manipuler des dates et heures"]', 120, 'Tu es un tuteur de programmation. Montre la documentation officielle Python. Fais créer des petits projets (dé, loterie, compte à rebours).', '["bibliothèques", "import", "math", "random", "datetime", "projets"]', 8, CURRENT_TIMESTAMP);

-- Unit 3: Digital Citizenship & Beyond / Citoyenneté numérique et au-delà (25 hours)
INSERT INTO modules (id, course_id, title, title_en, description, order_index, estimated_hours, created_at)
VALUES (
    'mod-icd20-u3',
    'course-icd20-2026',
    'Unité 3 : Citoyenneté numérique et au-delà',
    'Unit 3: Digital Citizenship & Beyond',
    'Ethics, privacy, cybersecurity basics, and emerging technologies.',
    3,
    25,
    CURRENT_TIMESTAMP
);

INSERT INTO lessons (id, module_id, title, title_en, description, topics, objectives, estimated_minutes, system_prompt, keywords, order_index, created_at) VALUES
('les-icd20-3-1', 'mod-icd20-u3', '3.1 — Éthique et propriété intellectuelle', '3.1 — Ethics & Intellectual Property', 'Copyright, open source, licensing, and digital rights.', '["Éthique", "Propriété intellectuelle", "Copyright", "Open source", "Licences"]', '["Expliquer le copyright et les licences", "Distinguer logiciel propriétaire et open source", "Respecter la propriété intellectuelle"]', 120, 'Tu es un mentor en informatique. Utilise des cas concrets (musique, images, code). Encourage la réflexion éthique.', '["éthique", "copyright", "open source", "licences", "propriété intellectuelle"]', 1, CURRENT_TIMESTAMP),
('les-icd20-3-2', 'mod-icd20-u3', '3.2 — Vie privée et sécurité', '3.2 — Privacy & Security', 'Passwords, phishing, encryption basics, and safe browsing.', '["Vie privée", "Sécurité", "Mots de passe", "Phishing", "Chiffrement"]', '["Créer des mots de passe robustes", "Reconnaître une tentative de phishing", "Expliquer le chiffrement de base"]', 120, 'Tu es un mentor en informatique. Fais des quiz interactifs (vrai/faux sur des scénarios). Donne des conseils pratiques.', '["vie privée", "sécurité", "mots de passe", "phishing", "chiffrement"]', 2, CURRENT_TIMESTAMP),
('les-icd20-3-3', 'mod-icd20-u3', '3.3 — Intelligence artificielle — Introduction', '3.3 — Artificial Intelligence — Introduction', 'What AI is, how it learns, and societal implications.', '["Intelligence artificielle", "Apprentissage automatique", "Biais", "Implications"]', '["Décrire ce qu''est l''IA et comment elle apprend", "Identifier des biais dans les données", "Évaluer les implications sociétales"]', 120, 'Tu es un mentor en informatique. Utilise des exemples accessibles (recommandations Netflix, reconnaissance faciale). Encourage la pensée critique.', '["IA", "apprentissage automatique", "biais", "implications", "critique"]', 3, CURRENT_TIMESTAMP);
