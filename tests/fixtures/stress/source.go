package stress

import "context"

type Loader[T any] interface {
	Load(context.Context, string) (T, error)
}

type Cache[T any] struct {
	loader Loader[T]
	values map[string]T
}

func NewCache[T any](loader Loader[T]) *Cache[T] {
	return &Cache[T]{loader: loader, values: map[string]T{}}
}

func (c *Cache[T]) Get(ctx context.Context, key string) (T, error) {
	if value, ok := c.values[key]; ok {
		return value, nil
	}
	value, err := c.loader.Load(ctx, key)
	if err != nil {
		var zero T
		return zero, err
	}
	c.values[key] = value
	return value, nil
}

func wrapLoader[T any](loader Loader[T]) func(context.Context, string) (T, error) {
	return func(ctx context.Context, key string) (T, error) {
		return loader.Load(ctx, key)
	}
}
