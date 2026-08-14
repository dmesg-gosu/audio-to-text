# Audio to Text Converter

Утилита командной строки для преобразования MP3 аудиофайлов в текст с использованием локальной модели Whisper и опциональной корректировкой через Mistral.

## Возможности

- ✅ Преобразование MP3 в текст локально (без облака)
- ✅ Использование модели Whisper (tiny, base, small, medium, large)
- ✅ Временные метки `(MM:SS)` перед каждой строкой текста
- ✅ Подсчет пауз между фразами с подробной статистикой
- ✅ Post-processing через Mistral для корректировки ошибок
- ✅ Live output транскрипции в реальном времени
- ✅ Простой CLI интерфейс
- ✅ Кроссплатформенность (Windows, macOS, Linux)

## Требования

- [OpenAI Whisper](https://github.com/openai/whisper) установлена локально
- Для корректировки: [Ollama](https://ollama.ai) с моделью Mistral
- Rust 1.70+ (если собираешь из исходников)

## Установка

### 1. Установи OpenAI Whisper

```bash
# Через pip
pip install openai-whisper

# Или через pipx (рекомендуется)
pipx install openai-whisper
```

### 2. (Опционально) Установи Ollama с Mistral

Для корректировки текста через Mistral:

```bash
# Установи Ollama с https://ollama.ai
ollama pull mistral
ollama serve
```

### 3. Собери программу

```bash
# Клонируй репозиторий
git clone https://github.com/dmesg-gosu/audio-to-text.git
cd audio-to-text

# Собери
cargo build --release

# Бинарник: target/release/audio-to-text
```

## Использование

### Базовое использование (один файл)

```bash
audio-to-text --input audio.mp3
```

Создаст файл `audio.txt` с временными метками и статистикой пауз.

### Batch-режим (обработка папки)

```bash
# Обработить все MP3 файлы в папке
audio-to-text --batch ./audio_files/

# С корректировкой через Mistral
audio-to-text --batch ./audio_files/ --correct

# С выбором модели
audio-to-text --batch ./audio_files/ --model medium --correct
```

Программа автоматически:
- Найдет все MP3 файлы в папке
- Обработает их по очереди
- Создаст TXT файлы рядом с каждым MP3
- Покажет прогресс (X из Y файлов)

### С корректировкой через Mistral

```bash
audio-to-text --input audio.mp3 --correct
```

Дополнительно исправит ошибки, грамматику и английские слова в кириллице.

### Выбрать модель Whisper

```bash
# tiny (39MB, самая быстрая)
audio-to-text --input audio.mp3 --model tiny

# base (140MB, быстрая)
audio-to-text --input audio.mp3 --model base

# small (440MB, хорошее качество)
audio-to-text --input audio.mp3 --model small

# medium (1.5GB, лучшее качество для русского) - рекомендуется
audio-to-text --input audio.mp3 --model medium

# large (2.9GB, максимальное качество, очень медленно)
audio-to-text --input audio.mp3 --model large
```

### Указать язык

```bash
# Русский
audio-to-text --input audio.mp3 --language ru

# Английский
audio-to-text --input audio.mp3 --language en

# Французский
audio-to-text --input audio.mp3 --language fr
```

## Параметры

| Параметр | Сокращение | Значение по умолчанию | Описание |
|----------|------------|----------------------|---------|
| `--input` | `-i` | - | Путь к входному MP3 файлу (обязательный) |
| `--output` | `-o` | input_name.txt | Путь к выходному TXT файлу |
| `--model` | `-m` | medium | Модель Whisper: tiny, base, small, medium, large |
| `--language` | `-l` | auto | Код языка (en, ru, fr и т.д.). Auto-detect если не указан |
| `--correct` | - | false | Корректировать текст через Mistral |

## Примеры

```bash
# Простая транскрипция одного файла
audio-to-text -i интервью.mp3

# Batch-обработка папки на ночь
audio-to-text --batch ~/DevOps/audio/ --model medium --correct

# С выбором языка
audio-to-text -i podcast.mp3 -l ru

# Все параметры для одного файла
audio-to-text \
  --input meeting.mp3 \
  --output результаты.txt \
  --model medium \
  --language ru \
  --correct

# Batch с большой моделью (лучшее качество, медленно)
audio-to-text --batch ./audio_files/ --model large --correct
```

## Выходной формат

Программа создает TXT файл с временными метками:

```
(00:00) Так, тогда у нас как обычно проходит беседа. Сначала человек рассказывает о себе,
(00:02) некую вводную часть, о том, какие у него там достижения были,
(00:08) что он делал на предыдущем месте, соответственно, а потом мы ставим свои вопросики.
(00:15) Давайте и это начнем так же.
```

После транскрипции выводится статистика:

```
📊 Analysis
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Total duration:    0h 3m 17s
  Total pause time:  0h 1m 20s
  Number of pauses:  4
  Avg pause length:  20.14s
  Pause ratio:       40.9%

  Pause timeline:
    Pause 1: 00:04 (8.52s)
    Pause 2: 00:21 (8.04s)
    Pause 3: 00:55 (31.00s)
    Pause 4: 02:41 (33.00s)
```

## Производительность

| Модель | Размер | Скорость | Качество |
|--------|--------|----------|----------|
| tiny | 39MB | ⚡⚡⚡ | ⭐ |
| base | 140MB | ⚡⚡ | ⭐⭐ |
| small | 440MB | ⚡ | ⭐⭐⭐ |
| **medium** | 1.5GB | 🐢 | ⭐⭐⭐⭐ |
| large | 2.9GB | 🐌 | ⭐⭐⭐⭐⭐ |

**Рекомендация:** Используй `medium` для лучшего баланса качества и скорости на русском языке.

## Лицензия

MIT

## Автор

[@gosu_ai](https://t.me/gosu_ai)
