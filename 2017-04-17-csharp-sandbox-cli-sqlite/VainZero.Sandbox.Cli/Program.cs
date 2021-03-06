﻿using System;
using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;
using System.Collections.Generic;
using System.Data.Entity;
using System.Data.Entity.ModelConfiguration.Conventions;
using System.Data.SQLite;
using System.Diagnostics;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using SQLite.CodeFirst;

namespace VainZero.Sandbox
{
    public class Person
    {
        [Key]
        [Column]
        public long Id { get; set; }

        [Column]
        [StringLength(512)]
        public string Name { get; set; }
    }

    public class MyContext
        : DbContext
    {
        public MyContext()
            : base(new SQLiteConnection(ConnectionString), contextOwnsConnection: true)
        {
            Database.Log = text => Debug.WriteLine(text);
        }

        #region ConnectionString
        const string dbPath = @"./database.sqlite";

        static SQLiteConnectionStringBuilder ConnectionStringBuilder =>
            new SQLiteConnectionStringBuilder
            {
                DataSource = dbPath,
                ForeignKeys = true,
                BinaryGUID = false,
            };

        static string ConnectionString { get; } =
            ConnectionStringBuilder.ConnectionString;
        #endregion

        #region OnModelCreating
        class MyDbInitializer
            : SqliteCreateDatabaseIfNotExists<MyContext>
        {
            public MyDbInitializer(DbModelBuilder mb)
                : base(mb, nullByteFileMeansNotExisting: true)
            {
            }

            protected override void Seed(MyContext context)
            {
                base.Seed(context);

                context.Set<Person>().Add(new Person() { Name = "John Doe" });
                context.Set<Person>().Add(new Person() { Name = "vain0" });
                context.SaveChanges();
            }
        }

        protected sealed override void OnModelCreating(DbModelBuilder mb)
        {
            Database.SetInitializer(new MyDbInitializer(mb));

            mb.Entity<Person>();
        }
        #endregion
    }

    public sealed class Program
    {
        public void Run()
        {
            using (var context = new MyContext())
            {
                foreach (var person in context.Set<Person>())
                {
                    Console.WriteLine("{0}", person.Name);
                }
            }
        }

        public static void Main(string[] args)
        {
            new Program().Run();
        }
    }
}
