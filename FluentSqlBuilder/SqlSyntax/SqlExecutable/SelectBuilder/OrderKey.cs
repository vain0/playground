using System.Collections.Generic;
using System.Linq;

namespace FluentSqlBuilder.SqlSyntax
{
    sealed class OrderKey
        : SqlPart
    {
        public SqlExpression<IScalar> Expression { get; }
        public OrderDirection Direction { get; }

        public OrderKey(SqlExpression<IScalar> expression, OrderDirection direction)
        {
            Expression = expression;
            Direction = direction;
        }

        #region Tokens
        IEnumerable<SqlToken> OrderKeywordTokens =>
            Direction == OrderDirection.Descending
            ? new[] { SqlToken.FromString("desc") }
            : Enumerable.Empty<SqlToken>();

        internal override IEnumerable<SqlToken> Tokens =>
            Expression.Tokens.Concat(OrderKeywordTokens);
        #endregion
    }
}
