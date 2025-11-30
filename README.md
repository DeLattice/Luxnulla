# elux Core

Простая и мощная оболочка для **xray-core** с автоматизацией настройки и управления.

## Что это?

elux Core — это Rust-приложение, которое упрощает работу с xray-core через:
- Автоматическую настройку конфигураций
- RESTful API для управления
- Группировку и организацию прокси
- Горячую перезагрузку без перезапуска
- Проверку и валидацию конфигов

## Установка

```bash
git clone https://github.com/DeLattice/elux.git
cd elux
cargo build --release
cargo run --bin server
```

## Использование

### Запуск сервера
```bash
cargo run --bin server
# Сервер запустится на http://localhost:3000
```

### API endpoints

**Управление xray:**
- `GET /xray/` - статус xray
- `POST /xray/on` - запустить xray
- `POST /xray/off` - остановить xray
- `GET /xray/outbounds` - получить конфигурации
- `POST /xray/outbounds` - применить новые конфигурации

**Управление группами:**
- `GET /groups/` - список всех групп
- `POST /groups/{name}` - создать группу
- `GET /groups/{name}` - получить группу
- `DELETE /groups/{name}` - удалить группу

### Примеры

Запустить xray:
```bash
curl -X POST http://localhost:3000/xray/on
```

Создать группу конфигураций:
```bash
curl -X POST http://localhost:3000/groups/my-servers \
  -H "Content-Type: application/json" \
  -d '[{...}]'
```

## Конфигурация

Файлы создаются автоматически в `~/.config/elux/`:
- `xray.json` - основная конфигурация xray
- `elux.kdl` - настройки приложения

## Roadmap

- [ ] **v1.1**: Автоматическая проверка конфигураций
- [ ] **v1.2**: Web интерфейс для управления
- [ ] **v1.3**: Поддержка Windows

## fds


## Лицензия

GPL-3.0 License
