﻿using FluentSqlBuilder.Provider.Fake;

namespace FluentSqlBuilder.Test
{
    public static class FakeDb
    {
        public static FakeDbProvider Provider { get; } =
            new FakeDbProvider();

        public static SqlBuilder Sql { get; } =
            new SqlBuilder(Provider);

        public static Employee Employee { get; } =
            new Employee(Sql);

        public static Department Department { get; } =
            new Department(Sql);
    }
}
