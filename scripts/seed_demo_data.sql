-- Script de datos de demostración para YuweLongo (Nasa Yuwe)
USE yuwelongo;

-- Limpiar tablas si se requiere re-ejecutar
SET FOREIGN_KEY_CHECKS = 0;
TRUNCATE TABLE favoritos;
TRUNCATE TABLE juego;
TRUNCATE TABLE preguntas_juego;
TRUNCATE TABLE palabras;
TRUNCATE TABLE niveles_juego;
TRUNCATE TABLE categorias;
TRUNCATE TABLE usuarios;
SET FOREIGN_KEY_CHECKS = 1;

-- 1. USUARIOS (admin@yuwelongo.com / admin123, usuario@yuwelongo.com / user123, carlos@yuwelongo.com / user123)
INSERT INTO usuarios (id_usuario, nombre, correo, contrasena, rol, estado, fecha_registro) VALUES
(1, 'Administrador Yuwe', 'admin@yuwelongo.com', '$2a$10$IfUMaxbZsuomTuLnYTcyZuiO2b3ip2xqXlsVLnpJUdNPa.TAn.Ce6', 'ADMIN', 'Activo', '2026-01-15'),
(2, 'Kiwe Thewala', 'usuario@yuwelongo.com', '$2a$10$CkiThNKrHDe19RkemkPMAuhQ0g3kY9Ekaf3eWLo76yUiwqCayVTF6', 'USUARIO', 'Activo', '2026-02-01'),
(3, 'Carlos Vitonás', 'carlos@yuwelongo.com', '$2a$10$CkiThNKrHDe19RkemkPMAuhQ0g3kY9Ekaf3eWLo76yUiwqCayVTF6', 'USUARIO', 'Activo', '2026-02-10');

-- 2. CATEGORÍAS
INSERT INTO categorias (id_categoria, nombre, imagen_url, descripcion, estado) VALUES
(1, 'Animales', 'https://images.unsplash.com/photo-1548767797-d8c844163c4c?w=500', 'Vocabulario relacionado con los animales y seres de la naturaleza en Nasa Yuwe', 'Activa'),
(2, 'Naturaleza y Territorio', 'https://images.unsplash.com/photo-1506744038136-46273834b3fb?w=500', 'Elementos de la Madre Tierra (Kiwe), ríos, montañas y astros', 'Activa'),
(3, 'Familia y Comunidad', 'https://images.unsplash.com/photo-1491438590914-bc09fcaaf77a?w=500', 'Miembros de la familia, parentesco y la comunidad Nasa', 'Activa'),
(4, 'Saludos y Expresiones', 'https://images.unsplash.com/photo-1517486808906-6ca8b3f04846?w=500', 'Saludos cotidianos, cortesía y expresiones comunes en el territorio', 'Activa');

-- 3. PALABRAS
INSERT INTO palabras (id_palabra, palabra_nasa, traduccion, frase_ejemplo, imagen_url, audio_url, fecha_creacion, id_categoria) VALUES
(1, 'Üus', 'Perro', 'Üus kwe’sx yatna u’ywe’sh (El perro cuida nuestra casa)', 'https://images.unsplash.com/photo-1543466835-00a7907e9de1?w=500', NULL, '2026-02-15 10:00:00', 1),
(2, 'Michi', 'Gato', 'Michi yu’we pyatsx (El gato maúlla suavemente)', 'https://images.unsplash.com/photo-1514888286974-6c03e2ca1dba?w=500', NULL, '2026-02-15 10:05:00', 1),
(3, 'Sxaw', 'Pájaro', 'Sxaw kape’jna wejx (El pájaro vuela alto en el cielo)', 'https://images.unsplash.com/photo-1444464666168-49d633b86797?w=500', NULL, '2026-02-15 10:10:00', 1),
(4, 'Iiçx', 'Caballo', 'Iiçx kiwe’sx kshawa’k (El caballo camina por la vereda)', 'https://images.unsplash.com/photo-1534447677768-be436bb09401?w=500', NULL, '2026-02-15 10:15:00', 1),
(5, 'Kiwe', 'Tierra / Territorio', 'Kiwe kwe’sx nasa yuwe (La tierra es de nuestro pueblo Nasa)', 'https://images.unsplash.com/photo-1464822759023-fed622ff2c3b?w=500', NULL, '2026-02-15 10:20:00', 2),
(6, 'Yu’', 'Agua / Río', 'Yu’ kwe’sx dxi’j wet (El agua del río es limpia y fresca)', 'https://images.unsplash.com/photo-1432405972618-c60b0225b8f9?w=500', NULL, '2026-02-15 10:25:00', 2),
(7, 'Sek', 'Sol', 'Sek yuwe we’we’s wetx (El sol ilumina la mañana)', 'https://images.unsplash.com/photo-1506744038136-46273834b3fb?w=500', NULL, '2026-02-15 10:30:00', 2),
(8, 'A’te', 'Luna', 'A’te thekwe yatsx (La luna llena alumbra la noche)', 'https://images.unsplash.com/photo-1532693322450-2cb5c511067d?w=500', NULL, '2026-02-15 10:35:00', 2),
(9, 'Tata', 'Papá / Padre', 'Tata kiwete puxnxi (Mi papá trabaja en el campo)', 'https://images.unsplash.com/photo-1492562080023-ab3db95bfbce?w=500', NULL, '2026-02-15 10:40:00', 3),
(10, 'Mama', 'Mamá / Madre', 'Mama tulte tu’k (Mi mamá siembra en la huerta)', 'https://images.unsplash.com/photo-1544005313-94ddf0286df2?w=500', NULL, '2026-02-15 10:45:00', 3),
(11, 'Kwe’sx', 'Nosotros / Nuestra gente', 'Kwe’sx nasa thegwe (Nosotros somos el pueblo Nasa)', 'https://images.unsplash.com/photo-1529156069898-49953e39b3ac?w=500', NULL, '2026-02-15 10:50:00', 3),
(12, 'Pya’s', 'Hola / Buenos días', 'Pya’s e’sx nasa yuwe (Buenos días a toda la comunidad)', 'https://images.unsplash.com/photo-1517486808906-6ca8b3f04846?w=500', NULL, '2026-02-15 10:55:00', 4);

-- 4. NIVELES DE JUEGO
INSERT INTO niveles_juego (id_nivel, nombre, descripcion, orden, estado) VALUES
(1, 'Nivel 1 - Básico', 'Aprende los animales y saludos esenciales en Nasa Yuwe', 1, 'Activo'),
(2, 'Nivel 2 - Intermedio', 'Descubre los elementos de la Madre Tierra y la naturaleza', 2, 'Activo'),
(3, 'Nivel 3 - Avanzado', 'Conoce el vocabulario de la familia y expresiones comunitarias', 3, 'Activo');

-- 5. PREGUNTAS DE JUEGO (4 opciones con respuesta_correcta y xp_valor)
INSERT INTO preguntas_juego (id_pregunta, pregunta_texto, opcion1, opcion2, opcion3, opcion4, respuesta_correcta, xp_valor, imagen_url, audio_url, id_palabra, id_nivel) VALUES
-- Nivel 1 (Animales y Saludos)
(1, '¿Cuál es la palabra en Nasa Yuwe para "Perro"? Üus', 'Üus', 'Michi', 'Sxaw', 'Iiçx', 'Üus', 10, 'https://images.unsplash.com/photo-1543466835-00a7907e9de1?w=500', NULL, 1, 1),
(2, '¿Cuál es la palabra en Nasa Yuwe para "Gato"? Michi', 'Michi', 'Üus', 'Sxaw', 'Sek', 'Michi', 10, 'https://images.unsplash.com/photo-1514888286974-6c03e2ca1dba?w=500', NULL, 2, 1),
(3, '¿Cuál es la palabra en Nasa Yuwe para "Pájaro"? Sxaw', 'Sxaw', 'Iiçx', 'Michi', 'Yu’', 'Sxaw', 10, 'https://images.unsplash.com/photo-1444464666168-49d633b86797?w=500', NULL, 3, 1),
(4, '¿Cómo se dice "Hola" o "Buenos días" en Nasa Yuwe? Pya’s', 'Pya’s', 'Tata', 'Mama', 'Sek', 'Pya’s', 10, 'https://images.unsplash.com/photo-1517486808906-6ca8b3f04846?w=500', NULL, 12, 1),

-- Nivel 2 (Naturaleza y Territorio)
(5, '¿Cómo se dice "Tierra o Territorio" en Nasa Yuwe? Kiwe', 'Kiwe', 'Yu’', 'Sek', 'A’te', 'Kiwe', 15, 'https://images.unsplash.com/photo-1464822759023-fed622ff2c3b?w=500', NULL, 5, 2),
(6, '¿Cuál es la palabra en Nasa Yuwe para "Agua o Río"? Yu’', 'Yu’', 'Kiwe', 'Sek', 'Sxaw', 'Yu’', 15, 'https://images.unsplash.com/photo-1432405972618-c60b0225b8f9?w=500', NULL, 6, 2),
(7, '¿Cómo se dice "Sol" en Nasa Yuwe? Sek', 'Sek', 'A’te', 'Kiwe', 'Yu’', 'Sek', 15, 'https://images.unsplash.com/photo-1506744038136-46273834b3fb?w=500', NULL, 7, 2),
(8, '¿Cómo se llama a la "Luna" en Nasa Yuwe? A’te', 'A’te', 'Sek', 'Sxaw', 'Michi', 'A’te', 15, 'https://images.unsplash.com/photo-1532693322450-2cb5c511067d?w=500', NULL, 8, 2),

-- Nivel 3 (Familia y Comunidad)
(9, '¿Cómo se dice "Papá / Padre" en Nasa Yuwe? Tata', 'Tata', 'Mama', 'Kwe’sx', 'Üus', 'Tata', 20, 'https://images.unsplash.com/photo-1492562080023-ab3db95bfbce?w=500', NULL, 9, 3),
(10, '¿Cómo se dice "Mamá / Madre" en Nasa Yuwe? Mama', 'Mama', 'Tata', 'Kwe’sx', 'Pya’s', 'Mama', 20, 'https://images.unsplash.com/photo-1544005313-94ddf0286df2?w=500', NULL, 10, 3),
(11, '¿Qué palabra significa "Nosotros / Nuestra gente"? Kwe’sx', 'Kwe’sx', 'Kiwe', 'Tata', 'Sek', 'Kwe’sx', 20, 'https://images.unsplash.com/photo-1529156069898-49953e39b3ac?w=500', NULL, 11, 3),
(12, '¿Cuál es la traducción de "Caballo" en Nasa Yuwe? Iiçx', 'Iiçx', 'Sxaw', 'Michi', 'Üus', 'Iiçx', 20, 'https://images.unsplash.com/photo-1534447677768-be436bb09401?w=500', NULL, 4, 3);

-- 6. FAVORITOS DE DEMOSTRACIÓN (Para usuario id 2)
INSERT INTO favoritos (id_favorito, fecha_guardado, id_usuario, id_palabra) VALUES
(1, '2026-03-01 11:00:00', 2, 1),
(2, '2026-03-01 11:05:00', 2, 5),
(3, '2026-03-01 11:10:00', 2, 12);

-- 7. PARTIDAS DE JUEGO (Historial y Ranking Global)
INSERT INTO juego (id_juego, puntaje, fecha_juego, preguntas_correctas, total_Preguntas, idPartida, id_usuario, id_nivel) VALUES
(1, 40, '2026-03-02 14:30:00', 4, 4, 'partida-demo-01', 2, 1),
(2, 60, '2026-03-02 15:10:00', 4, 4, 'partida-demo-02', 2, 2),
(3, 40, '2026-03-02 16:00:00', 4, 4, 'partida-demo-03', 3, 1),
(4, 45, '2026-03-02 16:45:00', 3, 4, 'partida-demo-04', 3, 2),
(5, 80, '2026-03-03 10:20:00', 4, 4, 'partida-demo-05', 1, 3),
(6, 60, '2026-03-03 11:00:00', 4, 4, 'partida-demo-06', 1, 2);
