#!/usr/bin/env bash
# Prueba de humo de todos los endpoints (Verification Plan de la migración).
# Requisitos: curl, jq y el servidor corriendo.
#   BASE=http://localhost:8080/api ./scripts/smoke_test.sh
set -u
BASE="${BASE:-http://localhost:8080/api}"
SUF="$(date +%s)"
OK=0; FAIL=0
BODY=$(mktemp)

# call METHOD PATH [JSON] [TOKEN]  -> deja el código HTTP en $CODE y el cuerpo en $BODY
call() {
  local method="$1" path="$2" data="${3:-}" token="${4:-}"
  local args=(-s -o "$BODY" -w '%{http_code}' -X "$method" "$BASE$path")
  [ -n "$data" ]  && args+=(-H 'Content-Type: application/json' -d "$data")
  [ -n "$token" ] && args+=(-H "Authorization: Bearer $token")
  CODE=$(curl "${args[@]}")
}
check() { # check "descripción" código_esperado
  if [ "$CODE" = "$2" ]; then OK=$((OK+1)); printf '  ok    %s (%s)\n' "$1" "$CODE"
  else FAIL=$((FAIL+1)); printf '  FALLA %s: esperaba %s y fue %s -> %s\n' "$1" "$2" "$CODE" "$(head -c 200 "$BODY")"; fi
}
jqb() { jq -r "$1" "$BODY"; }

echo "== Seguridad =="
call GET /usuarios;                         check "GET /usuarios sin token -> 401" 401
call GET /usuarios "" "token.invalido.x";   check "token inválido -> 401" 401
call GET /categorias;                       check "GET /categorias público" 200
call OPTIONS /usuarios;                     check "OPTIONS sin token" 200

echo "== Usuarios y login =="
EMAIL="smoke$SUF@test.com"
call POST /usuarios "{\"nombre\":\"Smoke\",\"correo\":\"$EMAIL\",\"contrasena\":\"secreto123\",\"rol\":\"ADMIN\"}"
check "POST /usuarios (registro público)" 200
call POST /usuarios "{\"nombre\":\"Smoke\",\"correo\":\"$EMAIL\",\"contrasena\":\"x\"}"
check "correo duplicado -> 409" 409
call POST /login "{\"correo\":\"$EMAIL\",\"contrasena\":\"mala\"}";  check "login con clave mala -> 401" 401
call POST /login "{\"correo\":\"$EMAIL\"}";                         check "login sin contraseña -> 400" 400
call POST /login "{\"correo\":\"$EMAIL\",\"contrasena\":\"secreto123\"}"; check "login correcto" 200
TOKEN=$(jqb .accessToken); ROL=$(jqb .rol)
[ -n "$TOKEN" ] && [ "$ROL" = "ADMIN" ] && echo "  ok    token recibido, rol=$ROL" || { echo "  FALLA sin token"; FAIL=$((FAIL+1)); }
call GET "/usuarios?correo=$EMAIL" "" "$TOKEN"; check "GET /usuarios?correo=" 200
UID_=$(jqb .idUsuario)
[ "$(jqb '.contrasena // "oculta"')" = "oculta" ] && echo "  ok    el hash no se expone" || { echo "  FALLA se expone el hash"; FAIL=$((FAIL+1)); }
call GET /usuarios "" "$TOKEN";                 check "GET /usuarios con token" 200
call GET "/usuarios/$UID_" "" "$TOKEN";         check "GET /usuarios/{id}" 200
call PUT /usuarios "{\"idUsuario\":$UID_,\"nombre\":\"Smoke 2\"}" "$TOKEN"; check "PUT /usuarios (id en cuerpo)" 200
call PUT "/usuarios/$UID_" "{\"estado\":\"Activo\"}" "$TOKEN";              check "PUT /usuarios/{id}" 200
call GET "/usuarios/$UID_" "" "$TOKEN"
[ "$(jqb .nombre)" = "Smoke 2" ] && [ "$(jqb .rol)" = "ADMIN" ] && echo "  ok    actualización parcial conserva rol" || { echo "  FALLA update parcial"; FAIL=$((FAIL+1)); }
call POST /logout "" "$TOKEN";                  check "POST /logout" 200
call POST /logout;                              check "POST /logout sin token -> 401" 401
call GET /usuarios/999999 "" "$TOKEN";          check "usuario inexistente -> 404" 404

echo "== Categorías =="
call POST /categorias '{"nombre":"Animales","descripcion":"Seres vivos","imagenUrl":"http://x/y.png"}' "$TOKEN"; check "POST /categorias" 201
call POST /categorias '{"nombre":"Sin token"}';  check "POST /categorias sin token -> 401" 401
call GET "/categorias?nombre=anim";              check "búsqueda por nombre (sin mayúsculas)" 200
CAT=$(jqb '.[0].idCategoria'); [ "$(jqb '.[0].estado')" = "Activa" ] && echo "  ok    estado forzado a Activa"
jqb '.[0].descripcion' | grep -q 'Creado:' && echo "  ok    descripción con fecha de creación"
call GET "/categorias/$CAT";                     check "GET /categorias/{id}" 200
call PUT "/categorias/$CAT" '{"nombre":"Animales 2","descripcion":"d","imagenUrl":"u"}' "$TOKEN"; check "PUT /categorias/{id}" 200

echo "== Palabras =="
call POST "/palabras?idCategoria=$CAT" '{"palabraNasa":"Üus","traduccion":"perro","fraseEjemplo":"Üus kwe’sx","imagenUrl":"i","audioUrl":"a"}' "$TOKEN"; check "POST /palabras" 201
call POST "/palabras?idCategoria=99999" '{"palabraNasa":"x"}' "$TOKEN";  check "categoría inexistente -> 404" 404
call GET /palabras;                              check "GET /palabras" 200
PAL=$(jqb '.[0].idPalabra'); [ "$(jqb '.[0].categoria.nombre')" = "Animales 2" ] && echo "  ok    categoría anidada en palabra"
[ "$(jqb '.[0].palabraNasa')" = "Üus" ] && echo "  ok    UTF-8 conservado"
call GET "/palabras?traduccion=PER";             check "búsqueda por traducción" 200
call GET "/palabras?palabraNasa=üu";             check "búsqueda por palabraNasa" 200
call GET "/palabras/categoria/$CAT";             check "palabras por categoría" 200
call GET "/palabras/categoria/99999";            check "categoría sin palabras -> 404" 404
call PUT "/palabras/$PAL?idCategoria=$CAT" '{"traduccion":"perro (canis)"}' "$TOKEN"; check "PUT /palabras/{id}" 200
call GET "/palabras/$PAL"; [ "$(jqb .traduccion)" = "perro (canis)" ] && [ "$(jqb .palabraNasa)" = "Üus" ] && echo "  ok    update parcial de palabra"

echo "== Favoritos =="
call POST "/favoritos?idUsuario=$UID_&idPalabra=$PAL" "" "$TOKEN"; check "POST /favoritos" 201
call POST "/favoritos?idUsuario=$UID_&idPalabra=$PAL" "" "$TOKEN"; check "favorito repetido -> 409" 409
call POST "/favoritos?idUsuario=$UID_&idPalabra=99999" "" "$TOKEN"; check "palabra inexistente -> 404" 404
call GET /favoritos "" "$TOKEN";                  check "GET /favoritos" 200
FAV=$(jqb '.[0].idFavorito')
call GET "/favoritos/$FAV" "" "$TOKEN";           check "GET /favoritos/{id}" 200
[ "$(jqb .palabra.categoria.nombre)" = "Animales 2" ] && echo "  ok    favorito -> palabra -> categoría anidados"
call GET "/favoritos/usuario/$UID_" "" "$TOKEN";  check "favoritos por usuario" 200

echo "== Niveles =="
call POST /niveles-juego '{"nombre":"Nivel 1","descripcion":"Fácil","orden":1}' "$TOKEN"; check "POST /niveles-juego" 200
NIV=$(jqb .idNivel); [ "$(jqb .estado)" = "Activo" ] && echo "  ok    estado por defecto Activo"
call POST /niveles-juego '{"nombre":"Nivel 0","orden":0,"estado":"Inactivo"}' "$TOKEN"; check "nivel inactivo" 200
NIV_OFF=$(jqb .idNivel)
call GET /niveles-juego; check "GET /niveles-juego" 200
[ "$(jqb 'map(.idNivel) | contains([-1]) | not')" = "true" ] && [ "$(jqb "map(select(.idNivel==$NIV_OFF)) | length")" = "0" ] && echo "  ok    solo niveles activos"
call GET "/niveles-juego/$NIV";                   check "GET /niveles-juego/{id}" 200
call GET "/niveles-juego/99999";                  check "nivel inexistente -> 204" 204
call PUT "/niveles-juego/$NIV" '{"nombre":"Nivel 1b","descripcion":"d","orden":2,"estado":"Activo"}' "$TOKEN"; check "PUT /niveles-juego/{id}" 200

echo "== Preguntas =="
Q='{"preguntaTexto":"¿Qué es Üus?","opcion1":"perro","opcion2":"gato","opcion3":"casa","opcion4":"agua","respuestaCoreccta":"perro","xpValor":10,"audio_url":"a.mp3","palabra":{"idPalabra":'$PAL'},"nivel":{"idNivel":'$NIV'}}'
call POST /preguntas-juego "$Q" "$TOKEN";          check "POST /preguntas-juego" 200
PRE=$(jqb .idPregunta)
[ "$(jqb .respuestaCoreccta)" = "perro" ] && [ "$(jqb .audio_url)" = "a.mp3" ] && [ "$(jqb .palabra.categoria.nombre)" = "Animales 2" ] && echo "  ok    nombres JSON de Java (respuestaCoreccta, audio_url) y anidados"
call POST /preguntas-juego '{"preguntaTexto":"x"}' "$TOKEN"; check "pregunta sin nivel -> 400" 400
call POST /preguntas-juego '{"nivel":{"idNivel":99999}}' "$TOKEN"; check "nivel inexistente -> 404" 404
for i in 2 3 4; do call POST /preguntas-juego "{\"preguntaTexto\":\"P$i\",\"nivel\":{\"idNivel\":$NIV}}" "$TOKEN"; done
call GET /preguntas-juego;                         check "GET /preguntas-juego" 200
call GET "/preguntas-juego/$PRE";                  check "GET /preguntas-juego/{id}" 200
call GET "/preguntas-juego/nivel/$NIV";            check "preguntas por nivel" 200
[ "$(jqb length)" = "4" ] && echo "  ok    4 preguntas en el nivel"
call GET "/preguntas-juego/nivel/$NIV?aleatorias=true&cantidad=2"; check "preguntas aleatorias" 200
[ "$(jqb length)" = "2" ] && echo "  ok    cantidad=2 respetada"
call PUT "/preguntas-juego/$PRE" '{"opcion2":"zorro","xpValor":25}' "$TOKEN"; check "PUT /preguntas-juego/{id}" 200
[ "$(jqb .opcion2)" = "zorro" ] && [ "$(jqb .opcion1)" = "perro" ] && [ "$(jqb .xpValor)" = "25" ] && echo "  ok    update parcial de pregunta"
call GET "/preguntas-juego/99999";                 check "pregunta inexistente -> 404" 404

echo "== Juegos =="
J1="{\"puntaje\":80,\"fechaJuego\":\"2026-10-24 10:00:00\",\"preguntasCorrectas\":8,\"totalPreguntas\":10,\"idPartida\":\"P-$SUF\",\"usuario\":{\"idUsuario\":$UID_},\"nivel\":{\"idNivel\":$NIV}}"
call POST /juegos "$J1" "$TOKEN";                  check "POST /juegos" 200
JID=$(jqb .idJuego); [ "$(jqb .usuario.correo)" = "$EMAIL" ] && echo "  ok    usuario y nivel anidados en juego"
call POST /juegos "$J1" "";                        check "POST /juegos sin token -> 401" 401
call POST /juegos "${J1/80/95}" "$TOKEN";          check "segunda partida" 200
call POST /juegos "{\"usuario\":{\"idUsuario\":99999}}" "$TOKEN"; check "usuario inexistente -> 400" 400
call PUT "/juegos/$JID" "${J1/80/85}" "$TOKEN";    check "PUT /juegos/{id}" 200
call GET "/juegos/historial/$UID_";                check "historial (público)" 200
[ "$(jqb length)" = "2" ] && echo "  ok    2 partidas en el historial"
call GET "/juegos/mejores/$NIV?limite=1";          check "mejores por nivel" 200
[ "$(jqb '.[0].puntaje')" = "95" ] && [ "$(jqb length)" = "1" ] && echo "  ok    mejor puntaje 95 y limite=1"
call GET "/juegos/mejor/$UID_/$NIV";               check "mejor puntaje de usuario" 200
call GET "/juegos/mejor/$UID_/99999";              check "sin partidas -> 204" 204
call GET "/juegos/total/$UID_";                    check "puntaje total" 200
[ "$(cat "$BODY")" = "180" ] && echo "  ok    total = 180 (número JSON)" || echo "  total=$(cat "$BODY")"
call GET "/juegos/ranking?idPartida=P-$SUF";       check "ranking por partida" 200
[ "$(jqb '.[0][2]')" = "180" ] && [ "$(jqb '.[0] | length')" = "3" ] && echo "  ok    ranking = [id, nombre, total]"
call GET "/juegos/ranking?limite=5";               check "ranking global" 200
[ "$(jqb '.[0] | length')" = "4" ] && echo "  ok    ranking global = [id, nombre, total, idPartida]"

echo "== Borrados =="
# Esquema propio (db.rs) tiene ON DELETE CASCADE -> 200. Una base creada por Java no -> 409.
call DELETE "/usuarios/$UID_" "" "$TOKEN"
if [ "$CODE" = "409" ]; then OK=$((OK+1)); echo "  ok    borrar usuario con partidas -> 409 (esquema sin cascadas, creado por Java)"
else check "borrar usuario con juegos/favoritos (cascada)" 200
     call GET "/usuarios/$UID_" "" "$TOKEN"; check "usuario ya no existe" 404; fi
call DELETE "/favoritos/$FAV" "" "$TOKEN"
call DELETE "/preguntas-juego/$PRE" "" "$TOKEN";   check "DELETE /preguntas-juego/{id}" 204
call DELETE "/niveles-juego/$NIV_OFF" "" "$TOKEN"; check "DELETE /niveles-juego/{id}" 204
call DELETE "/palabras/$PAL" "" "$TOKEN";          check "DELETE /palabras/{id}" 200
call DELETE "/categorias/$CAT" "" "$TOKEN";        check "DELETE /categorias/{id}" 200

echo
echo "Resultado: $OK correctas, $FAIL fallidas"
rm -f "$BODY"
[ "$FAIL" = "0" ]
