-- Almanach Seed Data: Ontario Grade 9 Mathematics (MTH1W)
-- Semester 1 / Semestre 1
-- Insert this after SCHEMA_COURSES tables exist.

-- Course
INSERT INTO courses (id, code, title, title_en, description, grade, language, credit_hours, created_at)
VALUES (
    'course-mth1w-2026',
    'MTH1W',
    'Mathématiques, 9e année (Décloisonné)',
    'Mathematics, Grade 9 (De-streamed)',
    'Ontario Grade 9 de-streamed mathematics covering Number Sense, Algebra, Data, Geometry & Measurement, and Financial Literacy.',
    '9',
    'fr',
    1,
    CURRENT_TIMESTAMP
);

-- Unit 1: Number Sense & Algebra / Sens du nombre et algèbre (25 hours)
INSERT INTO modules (id, course_id, title, title_en, description, order_index, estimated_hours, created_at)
VALUES (
    'mod-mth1w-u1',
    'course-mth1w-2026',
    'Unité 1 : Sens du nombre et algèbre',
    'Unit 1: Number Sense & Algebra',
    'Sets of numbers, exponents, fractions, ratios, algebraic expressions, equations, and introductory coding.',
    1,
    25,
    CURRENT_TIMESTAMP
);

INSERT INTO lessons (id, module_id, title, title_en, description, topics, objectives, estimated_minutes, system_prompt, keywords, order_index, created_at) VALUES
('les-mth1w-1-1', 'mod-mth1w-u1', '1.1 — Ensembles de nombres', '1.1 — Sets of Numbers', 'ℕ, ℤ, ℚ, ℝ — understanding the hierarchy of number sets.', '["Ensembles de nombres","Nombres naturels","Nombres entiers","Nombres rationnels","Nombres réels"]', '["Identifier les différents ensembles de nombres","Placer des nombres sur la droite numérique","Distinguer ℚ de ℝ"]',
120, 'Tu es un tuteur de mathématiques patient. Explique les ensembles de nombres en français simple. Utilise des analogies quotidiennes. Encourage l''élève à donner des exemples.', '["ensembles","naturels","entiers","rationnels","réels","droite numérique"]', 1, CURRENT_TIMESTAMP),
('les-mth1w-1-2', 'mod-mth1w-u1', '1.2 — Puissances et lois des exposants', '1.2 — Powers & Exponent Laws', 'Product rule, quotient rule, power of a power, zero and negative exponents.', '["Puissances","Lois des exposants","Exposant nul","Exposant négatif"]', '["Appliquer les lois des exposants","Simplifier des expressions avec puissances","Résoudre des problèmes avec des exposants négatifs"]',
180, 'Tu es un tuteur de mathématiques. Explique les lois des exposants étape par étape. Utilise des exemples numériques avant les variables. Félicite l''élève pour chaque bonne réponse.', '["exposants","puissances","lois","produit","quotient","négatif"]', 2, CURRENT_TIMESTAMP),
('les-mth1w-1-3', 'mod-mth1w-u1', '1.3 — Notation scientifique', '1.3 — Scientific Notation', 'Converting between standard and scientific notation, operations with scientific notation.', '["Notation scientifique","Puissances de 10","Conversion"]', '["Convertir entre notation standard et scientifique","Effectuer des opérations en notation scientifique","Comprendre l''échelle dans le monde réel"]',
120, 'Tu es un tuteur de mathématiques. Relie la notation scientifique à des exemples concrets (taille d''une cellule, distance Terre-Soleil). Utilise des comparaisons.', '["notation scientifique","puissances de 10","conversion","échelle"]', 3, CURRENT_TIMESTAMP),
('les-mth1w-1-4', 'mod-mth1w-u1', '1.4 — Fractions, ratios et taux', '1.4 — Fractions, Ratios & Rates', 'Operations with fractions, simplifying ratios, unit rates.', '["Fractions","Ratios","Taux","Proportions","Équivalents"]', '["Effectuer des opérations avec des fractions","Simplifier des ratios","Calculer des taux unitaires","Résoudre des problèmes de proportion"]',
180, 'Tu es un tuteur de mathématiques. Utilise des contextes familiers (recettes, sports, cartes). Sois patient avec la manipulation des fractions.', '["fractions","ratios","taux","proportions","simplification"]', 4, CURRENT_TIMESTAMP),
('les-mth1w-1-5', 'mod-mth1w-u1', '1.5 — Raisonnement proportionnel', '1.5 — Proportional Reasoning', 'Direct proportion, scaling, percent problems.', '["Proportion","Règle de trois","Pourcentage","Échelle"]', '["Résoudre des problèmes de proportion directe","Utiliser le produit croisé","Calculer des pourcentages dans des contextes réels"]',
120, 'Tu es un tuteur de mathématiques. Encourage le raisonnement proportionnel avec des problèmes concrets (réductions en magasin, cartes à l''échelle).', '["proportion","produit croisé","pourcentage","échelle","réduction"]', 5, CURRENT_TIMESTAMP),
('les-mth1w-1-6', 'mod-mth1w-u1', '1.6 — Expressions algébriques — Réduction', '1.6 — Algebraic Expressions — Collecting Like Terms', 'Variables, coefficients, like terms, simplifying expressions.', '["Expressions algébriques","Termes semblables","Réduction","Coefficients"]', '["Identifier les termes semblables","Réduire des expressions algébriques","Manipuler des expressions avec une seule variable"]',
120, 'Tu es un tuteur de mathématiques. Utilise des analogies visuelles (panier de fruits pour les termes semblables). Progression lentement des nombres aux variables.', '["expressions","termes semblables","coefficients","réduction","variables"]', 6, CURRENT_TIMESTAMP),
('les-mth1w-1-7', 'mod-mth1w-u1', '1.7 — Propriété distributive', '1.7 — Distributive Property', 'Expanding and factoring using the distributive property.', '["Distributivité","Factorisation","Développement","Parenthèses"]', '["Appliquer la propriété distributive","Développer des expressions","Factoriser des expressions simples"]',
120, 'Tu es un tuteur de mathématiques. Explique la distributivité avec des aires de rectangles. Montre les deux directions (développer et factoriser).', '["distributivité","factorisation","développement","parenthèses","aire"]', 7, CURRENT_TIMESTAMP),
('les-mth1w-1-8', 'mod-mth1w-u1', '1.8 — Résolution d''équations simples', '1.8 — Solving Simple Equations', 'One-step and two-step equations, balance method.', '["Équations","Isoler la variable","Opérations inverses","Balance"]', '["Résoudre des équations à une étape","Résoudre des équations à deux étapes","Vérifier une solution","Expliquer chaque étape du raisonnement"]',
180, 'Tu es un tuteur de mathématiques. Utilise la métaphore de la balance. Insiste sur la vérification de la solution. Encourage l''élève à expliquer ses étapes.', '["équations","isoler","opérations inverses","solution","vérification"]', 8, CURRENT_TIMESTAMP),
('les-mth1w-1-9', 'mod-mth1w-u1', '1.9 — Modélisation avec des équations', '1.9 — Modelling with Equations', 'Translating word problems into equations and solving.', '["Modélisation","Problèmes concrets","Traduction","Équations"]', '["Traduire un problème concret en équation","Résoudre et interpréter la solution","Vérifier si la solution a du sens dans le contexte"]',
120, 'Tu es un tuteur de mathématiques. Utilise des problèmes pertinents pour les adolescents (argent, temps, sports). Encourage la lecture attentive.', '["modélisation","problèmes","traduction","contexte","interprétation"]', 9, CURRENT_TIMESTAMP),
('les-mth1w-1-10', 'mod-mth1w-u1', '1.10 — Codage — Introduction au pseudocode', '1.10 — Coding — Intro to Pseudocode', 'Algorithms, sequences, decisions, loops in plain language.', '["Pseudocode","Algorithme","Séquence","Décision","Boucle"]', '["Écrire un algorithme en pseudocode","Utiliser des structures de contrôle","Décomposer un problème en étapes"]',
120, 'Tu es un tuteur d''informatique bienveillant. Explique que le code est juste une recette. Utilise des exemples très simples (faire un sandwich, se préparer le matin).', '["pseudocode","algorithme","séquence","décision","boucle","décomposition"]', 10, CURRENT_TIMESTAMP),
('les-mth1w-1-11', 'mod-mth1w-u1', '1.11 — Codage — Variables et sortie', '1.11 — Coding — Variables & Output', 'Declaring variables, assignment, output statements.', '["Variables","Affectation","Sortie","Types de données"]', '["Déclarer et utiliser des variables","Comprendre l''affectation","Produire une sortie formatée","Distinguer types de données"]',
120, 'Tu es un tuteur d''informatique. Explique les variables comme des boîtes étiquetées. Utilise Python ou pseudocode selon le confort de l''élève.', '["variables","affectation","sortie","types","python","pseudocode"]', 11, CURRENT_TIMESTAMP);

-- Unit 2: Data & Linear Relations / Données et relations linéaires (30 hours)
INSERT INTO modules (id, course_id, title, title_en, description, order_index, estimated_hours, created_at)
VALUES (
    'mod-mth1w-u2',
    'course-mth1w-2026',
    'Unité 2 : Données et relations linéaires',
    'Unit 2: Data & Linear Relations',
    'Data collection, analysis, scatter plots, linear relations, slope, equations of lines, and probability.',
    2,
    30,
    CURRENT_TIMESTAMP
);

INSERT INTO lessons (id, module_id, title, title_en, description, topics, objectives, estimated_minutes, system_prompt, keywords, order_index, created_at) VALUES
('les-mth1w-2-1', 'mod-mth1w-u2', '2.1 — Collecte et organisation de données', '2.1 — Collecting & Organizing Data', 'Primary vs secondary data, sampling methods, bias.', '["Collecte de données","Données primaires","Données secondaires","Échantillonnage","Biais"]', '["Distinguer données primaires et secondaires","Identifier des sources de biais","Choisir une méthode d''échantillonnage appropriée"]',
120, 'Tu es un tuteur de mathématiques. Utilise des exemples concrets (sondages à l''école, statistiques sportives). Parle de biais de manière critique mais accessible.', '["données","échantillonnage","biais","primaires","secondaires","sondage"]', 1, CURRENT_TIMESTAMP),
('les-mth1w-2-2', 'mod-mth1w-u2', '2.2 — Données à une variable — Moyenne, médiane, mode', '2.2 — One-Variable Data — Mean, Median, Mode', 'Measures of central tendency, range, outliers.', '["Moyenne","Médiane","Mode","Étendue","Valeurs aberrantes"]', '["Calculer la moyenne, la médiane et le mode","Choisir la meilleure mesure selon le contexte","Identifier et interpréter les valeurs aberrantes"]',
120, 'Tu es un tuteur de mathématiques. Explique quand utiliser chaque mesure (salaires → médiane, tailles → moyenne). Utilise des données réelles.', '["moyenne","médiane","mode","étendue","valeurs aberrantes","tendance centrale"]', 2, CURRENT_TIMESTAMP),
('les-mth1w-2-3', 'mod-mth1w-u2', '2.3 — Données à deux variables — Nuages de points', '2.3 — Two-Variable Data — Scatter Plots', 'Plotting ordered pairs, identifying patterns, correlation.', '["Nuage de points","Correlation","Variables","Coordonnées","Tendance"]', '["Créer un nuage de points à partir de données","Décrire la correlation (positive, négative, nulle)","Identifier des tendances dans des données réelles"]',
120, 'Tu es un tuteur de mathématiques. Utilise des données pertinentes (temps d''étude vs note, taille vs masse). Fais tracer l''élève mentalement.', '["nuage de points","corrélation","variables","tendance","coordonnées"]', 3, CURRENT_TIMESTAMP),
('les-mth1w-2-4', 'mod-mth1w-u2', '2.4 — Droites d''ajustement', '2.4 — Lines of Best Fit', 'Estimating lines of best fit, interpolation, extrapolation.', '["Droite d''ajustement","Interpolation","Extrapolation","Prédiction"]', '["Estimer une droite d''ajustement","Faire des prédictions par interpolation","Reconnaître les limites de l''extrapolation"]',
120, 'Tu es un tuteur de mathématiques. Utilise l''analogie du " fil tendu " entre des perles. Discute de la prudence nécessaire pour les prédictions.', '["droite d''ajustement","interpolation","extrapolation","prédiction","estimation"]', 4, CURRENT_TIMESTAMP),
('les-mth1w-2-5', 'mod-mth1w-u2', '2.5 — Introduction aux relations linéaires', '2.5 — Introduction to Linear Relations', 'Identifying linear patterns from tables and graphs.', '["Relation linéaire","Table de valeurs","Graphique","Taux de changement constant"]', '["Identifier une relation linéaire à partir d''un tableau","Représenter une relation linéaire graphiquement","Reconnaître un taux de changement constant"]',
120, 'Tu es un tuteur de mathématiques. Relie aux expériences concrètes (coût d''un taxi, remplissage d''une piscine). Montre la constance du taux.', '["relation linéaire","taux de changement","tableau","graphique","constante"]', 5, CURRENT_TIMESTAMP),
('les-mth1w-2-6', 'mod-mth1w-u2', '2.6 — Pente et taux de changement', '2.6 — Slope & Rate of Change', 'Rise over run, positive/negative/zero slope, steepness.', '["Pente","Taux de changement","Variation en y","Variation en x","Steepness"]', '["Calculer la pente à partir d''un graphique","Interpréter la pente dans un contexte","Distinguer pente positive, négative et nulle"]',
180, 'Tu es un tuteur de mathématiques. Utilise des contextes variés (vitesse, prix au litre, pente d''une côte). Relie la pente à la " vitesse " du changement.', '["pente","taux de changement","variation","graphique","contexte"]', 6, CURRENT_TIMESTAMP),
('les-mth1w-2-7', 'mod-mth1w-u2', '2.7 — Variation directe et partielle', '2.7 — Direct & Partial Variation', 'y = kx vs y = mx + b, initial value, constant rate.', '["Variation directe","Variation partielle","Valeur initiale","Ordonnée à l''origine"]', '["Distinguer variation directe et partielle","Identifier la valeur initiale","Exprimer une relation sous forme y = mx + b"]',
120, 'Tu es un tuteur de mathématiques. Utilise des comparaisons (location de vélo avec/without frais de base). Insiste sur l''ordonnée à l''origine.', '["variation directe","variation partielle","valeur initiale","ordonnée à l''origine","y = mx + b"]', 7, CURRENT_TIMESTAMP),
('les-mth1w-2-8', 'mod-mth1w-u2', '2.8 — Représentation graphique des relations linéaires', '2.8 — Graphing Linear Relations', 'Table, equation, graph connections. Finding intercepts.', '["Représentation graphique","Tableau","Équation","Intersections","Axe x","Axe y"]', '["Passer du tableau à l''équation au graphique","Trouver les intersections avec les axes","Choisir la meilleure représentation selon le besoin"]',
180, 'Tu es un tuteur de mathématiques. Fais créer les trois représentations par l''élève. Relie les intersections aux points spéciaux (départ, point de rentabilité).', '["graphique","tableau","équation","intersections","axes","représentations"]', 8, CURRENT_TIMESTAMP),
('les-mth1w-2-9', 'mod-mth1w-u2', '2.9 — Équations de droites (y = mx + b)', '2.9 — Equations of Lines (y = mx + b)', 'Finding slope and y-intercept from various representations.', '["Équation de droite","Pente","Ordonnée à l''origine","Forme y = mx + b"]', '["Déterminer la pente et l''ordonnée à l''origine","Écrire l''équation à partir d''un graphique","Tracer une droite à partir de son équation"]',
180, 'Tu es un tuteur de mathématiques. Fais beaucoup de pratique avec différentes entrées (graphique, deux points, pente et point). Insiste sur la signification des paramètres.', '["équation de droite","pente","ordonnée à l''origine","y = mx + b","tracé"]', 9, CURRENT_TIMESTAMP),
('les-mth1w-2-10', 'mod-mth1w-u2', '2.10 — Codage — Relations linéaires avec code', '2.10 — Coding — Linear Relations with Code', 'Writing code to generate tables and graphs of linear relations.', '["Codage","Relation linéaire","Tableau","Graphique","Python"]', '["Générer un tableau de valeurs avec du code","Créer un graphique simple avec code","Relier le code à l''équation y = mx + b"]',
120, 'Tu es un tuteur d''informatique. Utilise Python avec matplotlib ou des outils simples. Montre comment le code automatise ce que l''élève fait à la main.', '["codage","python","relation linéaire","graphique","automatisation"]', 10, CURRENT_TIMESTAMP),
('les-mth1w-2-11', 'mod-mth1w-u2', '2.11 — Modélisation mathématique — Modèles linéaires', '2.11 — Mathematical Modelling — Linear Models', 'Using linear models to predict and analyze real-world situations.', '["Modélisation","Modèle linéaire","Prédiction","Analyse","Contexte réel"]', '["Construire un modèle linéaire à partir de données","Utiliser le modèle pour prédire","Évaluer la qualité du modèle"]',
180, 'Tu es un tuteur de mathématiques. Utilise des données collectées par l''élève ou des données ouvertes (Environnement Canada, Statistique Canada).', '["modélisation","modèle linéaire","prédiction","données réelles","analyse"]', 11, CURRENT_TIMESTAMP),
('les-mth1w-2-12', 'mod-mth1w-u2', '2.12 — Notions de probabilité', '2.12 — Probability Basics', 'Simple probability, sample space, favourable outcomes.', '["Probabilité","Espace échantillonnal","Résultats favorables","Fréquence relative"]', '["Calculer une probabilité simple","Décrire l''espace échantillonnal","Relier probabilité et fréquence relative"]',
120, 'Tu es un tuteur de mathématiques. Utilise des exemples concrets (dé, pièce, tirage dans un sac). Fais expérimenter mentalement l''élève.', '["probabilité","espace échantillonnal","résultats favorables","fréquence relative","expérience"]', 12, CURRENT_TIMESTAMP),
('les-mth1w-2-13', 'mod-mth1w-u2', '2.13 — Probabilité expérimentale vs théorique', '2.13 — Experimental vs Theoretical Probability', 'Running simulations, law of large numbers.', '["Probabilité expérimentale","Probabilité théorique","Simulation","Loi des grands nombres"]', '["Distinguer probabilité expérimentale et théorique","Effectuer une simulation","Comprendre la loi des grands nombres"]',
120, 'Tu es un tuteur de mathématiques. Propose des simulations simples (lancer une pièce 100 fois virtuellement). Relie à la stabilisation des fréquences.', '["probabilité expérimentale","probabilité théorique","simulation","loi des grands nombres","fréquence"]', 13, CURRENT_TIMESTAMP);

-- Unit 3: Geometry & Measurement / Géométrie et mesure (30 hours)
INSERT INTO modules (id, course_id, title, title_en, description, order_index, estimated_hours, created_at)
VALUES (
    'mod-mth1w-u3',
    'course-mth1w-2026',
    'Unité 3 : Géométrie et mesure',
    'Unit 3: Geometry & Measurement',
    'Measurement systems, area, volume, Pythagorean theorem, properties of shapes, similarity, transformations.',
    3,
    30,
    CURRENT_TIMESTAMP
);

INSERT INTO lessons (id, module_id, title, title_en, description, topics, objectives, estimated_minutes, system_prompt, keywords, order_index, created_at) VALUES
('les-mth1w-3-1', 'mod-mth1w-u3', '3.1 — Systèmes de mesure (métrique / impérial)', '3.1 — Measurement Systems (Metric / Imperial)', 'Converting between metric and imperial units.', '["Système métrique","Système impérial","Conversion","Unités de longueur","Unités de masse"]', '["Convertir entre unités métriques","Convertir entre unités impériales","Passer d''un système à l''autre","Choisir l''unité appropriée"]',
120, 'Tu es un tuteur de mathématiques. Utilise des contextes canadiens (routes en km, construction en pieds/pouces). Donne des points de repère mémorables.', '["système métrique","système impérial","conversion","unités","canada"]', 1, CURRENT_TIMESTAMP),
('les-mth1w-3-2', 'mod-mth1w-u3', '3.2 — Conversions d''unités — Méthode du facteur', '3.2 — Unit Conversions — Factor-Label Method', 'Dimensional analysis for complex conversions.', '["Méthode du facteur","Analyse dimensionnelle","Conversion complexe","Chaîne de conversion"]', '["Utiliser la méthode du facteur-étiquette","Effectuer des conversions en plusieurs étapes","Vérifier l''homogénéité des unités"]',
120, 'Tu es un tuteur de mathématiques. Explique la méthode comme une " destruction " d''unités (comme au Scrabble). Sois très méthodique.', '["méthode du facteur","analyse dimensionnelle","conversion","homogénéité","étiquette"]', 2, CURRENT_TIMESTAMP),
('les-mth1w-3-3', 'mod-mth1w-u3', '3.3 — Périmètre et aire des figures 2D', '3.3 — Perimeter & Area of 2D Shapes', 'Triangles, quadrilaterals, circles, composite figures.', '["Périmètre","Aire","Triangle","Quadrilatère","Cercle","Figure composite"]', '["Calculer le périmètre de figures 2D","Calculer l''aire de figures usuelles","Décomposer une figure composite","Choisir la bonne formule"]',
180, 'Tu es un tuteur de mathématiques. Utilise des figures réelles (plan de maison, jardin). Fais dessiner l''élève pour décomposer.', '["périmètre","aire","triangle","quadrilatère","cercle","composite","formule"]', 3, CURRENT_TIMESTAMP),
('les-mth1w-3-4', 'mod-mth1w-u3', '3.4 — Théorème de Pythagore', '3.4 — Pythagorean Theorem', 'Proof, applications, converse, distance formula.', '["Théorème de Pythagore","Hypoténuse","Côtés","Preuve","Distance"]', '["Énoncer et appliquer le théorème de Pythagore","Vérifier si un triangle est rectangle","Calculer la distance entre deux points","Comprendre une preuve visuelle"]',
180, 'Tu es un tuteur de mathématiques. Montre la preuve par réarrangement des carrés. Relie à la distance dans le plan cartésien. Utilise des contextes (échelle, navigation).', '["Pythagore","hypoténuse","côtés","preuve","distance","triangle rectangle"]', 4, CURRENT_TIMESTAMP),
('les-mth1w-3-5', 'mod-mth1w-u3', '3.5 — Volume des prismes et cylindres', '3.5 — Volume of Prisms & Cylinders', 'V = Base × height, applications.', '["Volume","Prisme","Cylindre","Aire de la base","Hauteur"]', '["Calculer le volume d''un prisme droit","Calculer le volume d''un cylindre","Résoudre des problèmes de volume","Manipuler des unités cubiques"]',
180, 'Tu es un tuteur de mathématiques. Utilise des objets concrets (boîte de jus, boîte de conserve). Relie l''aire de la base au volume empilé.', '["volume","prisme","cylindre","aire de base","hauteur","unités cubiques"]', 5, CURRENT_TIMESTAMP),
('les-mth1w-3-6', 'mod-mth1w-u3', '3.6 — Volume des pyramides et cônes', '3.6 — Volume of Pyramids & Cones', 'V = (1/3) × Base × height, relationship to prisms.', '["Volume","Pyramide","Cône","Rapport 1/3","Comparaison"]', '["Calculer le volume d''une pyramide","Calculer le volume d''un cône","Établir le rapport 1:3 avec le prisme/cylindre","Résoudre des problèmes contextuels"]',
180, 'Tu es un tuteur de mathématiques. Utilise la démonstration avec du sable/eau (pyramide dans un prisme). L''élève retiendra le 1/3 grâce à l''image.', '["volume","pyramide","cône","rapport 1/3","comparaison","sable"]', 6, CURRENT_TIMESTAMP),
('les-mth1w-3-7', 'mod-mth1w-u3', '3.7 — Rapport : prisme vs pyramide, cylindre vs cône', '3.7 — Relationship: Prism vs Pyramid, Cylinder vs Cone', 'Comparing volumes, scaling effects.', '["Rapport de volumes","Effet d''échelle","Prisme vs pyramide","Cylindre vs cône"]', '["Comprendre le rapport 1:3 entre prisme et pyramide","Analyser l''effet du changement de dimensions","Généraliser à d''autres solides"]',
120, 'Tu es un tuteur de mathématiques. Utilise des visualisations 3D si possible. Demande à l''élève de prédire avant de calculer.', '["rapport","échelle","prisme","pyramide","cylindre","cône","prédiction"]', 7, CURRENT_TIMESTAMP),
('les-mth1w-3-8', 'mod-mth1w-u3', '3.8 — Aire de la surface des solides 3D', '3.8 — Surface Area of 3D Shapes', 'Nets, surface area of prisms, pyramids, cylinders, cones.', '["Aire de la surface","Développement","Solide 3D","Prisme","Pyramide","Cylindre","Cône"]', '["Représenter le développement d''un solide","Calculer l''aire de la surface","Distinguer aire de la surface et volume","Résoudre des problèmes d''emballage/construction"]',
180, 'Tu es un tuteur de mathématiques. Utilise des dépliages (nets) virtuels ou réels. Relie à l''emballage, la peinture, le revêtement.', '["aire de surface","développement","solide 3D","emballage","peinture","revêtement"]', 8, CURRENT_TIMESTAMP),
('les-mth1w-3-9', 'mod-mth1w-u3', '3.9 — Angles et droites parallèles', '3.9 — Angles & Parallel Lines', 'Corresponding, alternate, co-interior angles.', '["Angles correspondants","Angles alternes","Angles internes du même côté","Droites parallèles","Transversale"]', '["Identifier les paires d''angles formées par une transversale","Utiliser les propriétés des droites parallèles","Résoudre des problèmes avec des angles inconnus"]',
120, 'Tu es un tuteur de mathématiques. Utilise des schémas clairs avec codage des angles. Fais trouver les relations par l''élève avec des mesures données.', '["angles correspondants","angles alternes","droites parallèles","transversale","internes"]', 9, CURRENT_TIMESTAMP),
('les-mth1w-3-10', 'mod-mth1w-u3', '3.10 — Propriétés des triangles et quadrilatères', '3.10 — Properties of Triangles & Quadrilaterals', 'Sum of angles, classification, special quadrilaterals.', '["Triangles","Quadrilatères","Somme des angles","Classification","Propriétés"]', '["Classer les triangles selon les côtés et les angles","Classer les quadrilatères","Utiliser la somme des angles (180° et 360°)","Justifier à l''aide de propriétés"]',
120, 'Tu es un tuteur de mathématiques. Utilise des hiérarchies visuelles (arbre de classification). Fais construire des figures à la règle et au compas.', '["triangles","quadrilatères","classification","somme des angles","propriétés","construction"]', 10, CURRENT_TIMESTAMP),
('les-mth1w-3-11', 'mod-mth1w-u3', '3.11 — Similitude et rapports d''homothétie', '3.11 — Similarity & Scale Factors', 'Enlargements, reductions, ratio of sides, area ratio.', '["Similitude","Homothétie","Rapport d''échelle","Agrandissement","Réduction","Rapport d''aires"]', '["Reconnaître des figures semblables","Utiliser le rapport d''échelle","Calculer des dimensions dans un agrandissement/réduction","Relier rapport linéaire et rapport d''aire"]',
120, 'Tu es un tuteur de mathématiques. Utilise des cartes, des photocopies, des maquettes. Insiste sur le fait que le rapport d''aire est le carré du rapport linéaire.', '["similitude","échelle","agrandissement","réduction","rapport d''aires","carte"]', 11, CURRENT_TIMESTAMP),
('les-mth1w-3-12', 'mod-mth1w-u3', '3.12 — Transformations (translations, réflexions, rotations)', '3.12 — Transformations (Translations, Reflections, Rotations)', 'Describing and performing transformations on the coordinate plane.', '["Translation","Réflexion","Rotation","Plan cartésien","Image","Préimage"]', '["Effectuer une transformation sur le plan cartésien","Décrire une transformation (vecteur, axe, angle)","Reconnaître la conservation des mesures","Identifier des symétries"]',
180, 'Tu es un tuteur de mathématiques. Utilise du papier quadrillé ou des outils interactifs. Fais manipuler l''élève mentalement puis sur papier.', '["translation","réflexion","rotation","plan cartésien","symétrie","image","préimage"]', 12, CURRENT_TIMESTAMP);

-- Unit 4: Financial Literacy / Littératie financière (25 hours)
INSERT INTO modules (id, course_id, title, title_en, description, order_index, estimated_hours, created_at)
VALUES (
    'mod-mth1w-u4',
    'course-mth1w-2026',
    'Unité 4 : Littératie financière',
    'Unit 4: Financial Literacy',
    'Budgeting, simple and compound interest, appreciation/depreciation, credit, loans, mortgages, financial decision making.',
    4,
    25,
    CURRENT_TIMESTAMP
);

INSERT INTO lessons (id, module_id, title, title_en, description, topics, objectives, estimated_minutes, system_prompt, keywords, order_index, created_at) VALUES
('les-mth1w-4-1', 'mod-mth1w-u4', '4.1 — Décisions financières — Besoins vs envies', '4.1 — Financial Decisions — Needs vs Wants', 'Distinguishing essential and discretionary spending.', '["Besoins","Envies","Décision financière","Budget","Priorisation"]', '["Distinguer besoins et envies","Prioriser des dépenses","Justifier des choix financiers","Reconnaître des influences sur la consommation"]',
120, 'Tu es un tuteur de mathématiques. Utilise des scénarios réels (adolescent avec un premier emploi). Fais réfléchir l''élève sur ses propres choix sans juger.', '["besoins","envies","décision","budget","priorisation","consommation"]', 1, CURRENT_TIMESTAMP),
('les-mth1w-4-2', 'mod-mth1w-u4', '4.2 — Principes de budgétisation', '4.2 — Budgeting Basics', 'Income, expenses, surplus, deficit, budget formats.', '["Budget","Revenus","Dépenses","Excédent","Déficit","Épargne"]', '["Élaborer un budget simple","Distinguer revenus fixes et variables","Planifier l''épargne","Ajuster un budget en déséquilibre"]',
180, 'Tu es un tuteur de mathématiques. Utilise des données réalistes (salaire minimum Ontario, loyer moyen). Fais créer un budget par l''élève.', '["budget","revenus","dépenses","excédent","déficit","épargne","salaire minimum"]', 2, CURRENT_TIMESTAMP),
('les-mth1w-4-3', 'mod-mth1w-u4', '4.3 — Intérêt simple', '4.3 — Simple Interest', 'I = Prt, calculating interest and total amount.', '["Intérêt simple","Capital","Taux","Temps","Montant total"]', '["Calculer l''intérêt simple","Calculer le montant total","Comparer différents taux","Résoudre des problèmes de prêt à intérêt simple"]',
120, 'Tu es un tuteur de mathématiques. Utilise des exemples concrets (prêt entre amis, compte d''épargne). Montre la linéarité de l''intérêt simple.', '["intérêt simple","capital","taux","temps","montant total","prêt"]', 3, CURRENT_TIMESTAMP),
('les-mth1w-4-4', 'mod-mth1w-u4', '4.4 — Intérêt composé', '4.4 — Compound Interest', 'A = P(1 + r)^t, comparing simple vs compound, rule of 72.', '["Intérêt composé","Capitalisation","Puissance","Règle de 72","Comparaison"]', '["Calculer l''intérêt composé","Comparer intérêt simple et composé","Utiliser la règle de 72","Comprendre l''effet du temps sur la capitalisation"]',
180, 'Tu es un tuteur de mathématiques. Montre la " magie " de la capitalisation avec des tableaux. Compare à long terme (20-30 ans). Utilise des calculateurs.', '["intérêt composé","capitalisation","règle de 72","comparaison","temps","magie"]', 4, CURRENT_TIMESTAMP),
('les-mth1w-4-5', 'mod-mth1w-u4', '4.5 — Appréciation et dépréciation', '4.5 — Appreciation & Depreciation', 'Linear and exponential decay/growth in value.', '["Appréciation","Dépréciation","Valeur résiduelle","Amortissement","Croissance","Décroissance"]', '["Calculer la dépréciation linéaire","Calculer l''appréciation","Estimer une valeur résiduelle","Relier à des contextes (auto, immobilier, technologie)"]',
120, 'Tu es un tuteur de mathématiques. Utilise des exemples pertinents (voiture, téléphone, immobilier). Fais calculer la perte de valeur par année.', '["appréciation","dépréciation","valeur résiduelle","amortissement","auto","immobilier"]', 5, CURRENT_TIMESTAMP),
('les-mth1w-4-6', 'mod-mth1w-u4', '4.6 — Cartes de crédit et prêts', '4.6 — Credit Cards & Loans', 'How credit works, minimum payments, interest traps.', '["Carte de crédit","Prêt","Paiement minimum","Intérêts","Dette","Solde"]', '["Expliquer le fonctionnement d''une carte de crédit","Calculer le coût réel d''un achat à crédit","Reconnaître les pièges de la dette","Comparer différentes options de financement"]',
120, 'Tu es un tuteur de mathématiques. Utilise des exemples choquants mais réalistes (achat de 500$ payé au minimum → 3 ans, 800$). Sois informatif, pas moralisateur.', '["carte de crédit","prêt","paiement minimum","intérêts","dette","coût réel"]', 6, CURRENT_TIMESTAMP),
('les-mth1w-4-7', 'mod-mth1w-u4', '4.7 — Mise de fonds et hypothèques (contexte canadien)', '4.7 — Down Payments & Mortgages (Canadian Context)', 'CMHC, amortization, fixed vs variable rates.', '["Hypothèque","Mise de fonds","Amortissement","Taux fixe","Taux variable","SCHL"]', '["Calculer une mise de fonds","Comprendre l''amortissement","Distinguer taux fixe et variable","Estimer un paiement mensuel hypothécaire"]',
180, 'Tu es un tuteur de mathématiques. Utilise des données canadiennes actuelles (prix moyen des maisons, taux hypothécaires). Montre l''impact de la mise de fonds.', '["hypothèque","mise de fonds","amortissement","taux fixe","taux variable","schl","canada"]', 7, CURRENT_TIMESTAMP),
('les-mth1w-4-8', 'mod-mth1w-u4', '4.8 — Adapter un budget à des changements de situation', '4.8 — Adapting Budgets to Changing Circumstances', 'Job loss, unexpected expenses, windfalls, life changes.', '["Changement de situation","Dépense imprévue","Perte d''emploi","Gain inattendu","Adaptation"]', '["Modifier un budget face à un changement","Planifier pour les imprévus","Prioriser en période de restriction","Reconnaître l''importance de l''épargne d''urgence"]',
120, 'Tu es un tuteur de mathématiques. Utilise des scénarios réels (perte d''emploi, réparation de voiture, prime). Fais réagir l''élève sans juger.', '["changement","imprévu","perte d''emploi","épargne d''urgence","adaptation","priorisation"]', 8, CURRENT_TIMESTAMP),
('les-mth1w-4-9', 'mod-mth1w-u4', '4.9 — Comparer des produits financiers', '4.9 — Comparing Financial Products', 'GICs, savings accounts, TFSA, RESP basics.', '["Produits financiers","CÉLI","REEE","Compte d''épargne","Certificat de placement","Comparaison"]', '["Comparer différents produits d''épargne","Comprendre le CÉLI et le REEE","Calculer le rendement d''un placement","Choisir un produit adapté à un objectif"]',
120, 'Tu es un tuteur de mathématiques. Explique les produits canadiens simplement. Utilise des comparatifs (tableaux). Mentionne que les détails changent, donc vérifier.', '["produits financiers","céli","reee","épargne","placement","rendement","canada"]', 9, CURRENT_TIMESTAMP),
('les-mth1w-4-10', 'mod-mth1w-u4', '4.10 — Établissement d''objectifs financiers', '4.10 — Financial Goal Setting', 'Short, medium, long-term goals, SMART criteria.', '["Objectifs financiers","Court terme","Moyen terme","Long terme","Critères SMART","Plan d''action"]', '["Formuler des objectifs financiers SMART","Élaborer un plan d''action","Relier objectifs à des montants et échéances","Réviser des objectifs périodiquement"]',
120, 'Tu es un tuteur de mathématiques. Fais établir des objectifs réels à l''élève (première voiture, études, voyage). Sois encourageant et concret.', '["objectifs","SMART","court terme","moyen terme","long terme","plan d''action","concret"]', 10, CURRENT_TIMESTAMP);

-- Teacher insights types (enum reference — not a table, for documentation)
-- insight_type: 'exploring', 'creative_detour', 'deep_dive', 'struggling', 'on_track'
