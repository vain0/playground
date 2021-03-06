using System;
using System.Collections.Generic;

namespace Structures
{
    /// <summary>
    /// Represents a value to be generated by a supsended computation.
    /// </summary>
    public struct Susp<T>
    {
        /// <summary>
        /// Null if eager.
        /// </summary>
        Lazy<T> Lazy { get; }

        /// <summary>
        /// Is valid only if <c>Lazy</c> is null.
        /// </summary>
        T Eager { get; }

        public bool IsValueCreated =>
            Lazy == null || Lazy.IsValueCreated;

        public T Value =>
            Lazy == null ? Eager : Lazy.Value;

        internal Susp(T eager, Lazy<T> lazy)
        {
            Eager = eager;
            Lazy = lazy;
        }

        public bool Try(out T value)
        {
            if (IsValueCreated)
            {
                value = Value;
                return true;
            }

            value = default(T);
            return false;
        }

        public Susp<Y> Map<Y>(Func<T, Y> f)
        {
            if (Try(out var value))
            {
                return Susp.Eager(f(value));
            }

            var localLazy = Lazy;
            return Susp.Lazy(() => f(localLazy.Value));
        }

        public Susp<Y> FlatMap<Y>(Func<T, Susp<Y>> f)
        {
            if (Try(out var value))
            {
                return f(value);
            }

            var localLazy = Lazy;
            return Susp.Lazy(() => f(localLazy.Value).Value);
        }
    }

    public static class Susp
    {
        public static Susp<T> Eager<T>(T value) =>
            new Susp<T>(value, default(Lazy<T>));

        public static Susp<T> Lazy<T>(Func<T> compute) =>
            new Susp<T>(default(T), new Lazy<T>(compute, isThreadSafe: true));
    }
}
