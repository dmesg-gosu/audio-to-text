# Audio to Text Converter

Утилита командной строки для преобразования MP3 аудиофайлов в текст с использованием локальной модели Whisper через Ollama.

## Возможности

- Преобразование MP3 в текст локально (без облака)
- Использование модели Whisper через Ollama
- Простой CLI интерфейс
- Кроссплатформенность (Windows, macOS, Linux)

## Требования

- [Ollama](https://ollama.ai) установлена и запущена
- Модель Whisper загружена в Ollama
- Rust 1.70+ (если собираешь из исходников)

## Установка Ollama

### Windows и macOS

1. Скачай установщик с [ollama.ai](https://ollama.ai)
2. Установи приложение
3. Ollama автоматически запустится и будет слушать на `http://localhost:11434`

### Linux (Ubuntu/Debian)

```bash
# Загрузи и установи Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Проверь установку
ollama --version

# Запусти Ollama (если не запустилась автоматически)
ollama serve &
```

## Скачивание модели Whisper

После установки Ollama скачай модель Whisper:

```bash
ollama pull whisper
```

Это займёт 5-10 минут в зависимости от скорости интернета. Модель будет сохранена локально (~3GB).

## Установка программы

### Способ 1: Загрузить готовый бинарник (проще)

Скачай готовый `.exe` файл из [Releases](https://github.com/yourusername/audio-to-text/releases) и положи его в удобную папку.

### Способ 2: Собрать из исходников

#### Требования
- [Rust](https://rustup.rs/) установлен

#### Сборка

```bash
# Клонируй репозиторий
git clone https://github.com/yourusername/audio-to-text.git
cd audio-to-text

# Собери программу
cargo build --release

# Бинарник будет в target/release/audio-to-text (или audio-to-text.exe на Windows)
```

## Использование

### Базовое использование

```bash
audio-to-text --input audio.mp3
```

Программа автоматически создаст файл `audio.txt` с результатом.

### С указанием пути для вывода

```bash
audio-to-text --input audio.mp3 --output результат.txt
```

### Если Ollama на другом адресе

```bash
audio-to-text --input audio.mp3 --ollama http://192.168.1.100:11434
```

### Со всеми параметрами

```bash
audio-to-text \
  --input audio.mp3 \
  --output результат.txt \
  --ollama http://localhost:11434 \
  --model whisper
```

## Параметры

| Параметр | Сокращение | Значение по умолчанию | Описание |
|----------|------------|----------------------|---------|
| `--input` | `-i` | - | Путь к входному MP3 файлу (обязательный) |
| `--output` | `-o` | input_name.txt | Путь к выходному TXT файлу |
| `--ollama` | `-o` | http://localhost:11434 | URL Ollama API |
| `--model` | `-m` | whisper | Модель для транскрипции |

## Примеры

```bash
# Простая транскрипция
audio-to-text -i доклад.mp3

# С кастомным путем вывода
audio-to-text -i meeting.mp3 -o meeting_notes.txt

# Удаленный Ollama сервер
audio-to-text -i audio.mp3 --ollama http://server.example.com:11434
```

## Троблшутинг

### "connection refused"
Убедись, что Ollama запущена:
```bash
ollama serve
```

### "model not found: whisper"
Скачай модель:
```bash
ollama pull whisper
```

### Медленная транскрипция
- Это нормально для первого запуска (модель загружается в памяти)
- Убедись, что достаточно RAM (~4GB минимум)
- Проверь загруженность процессора

## Лицензия

MIT

## Автор

[@gosu_ai](https://t.me/gosu_ai)
