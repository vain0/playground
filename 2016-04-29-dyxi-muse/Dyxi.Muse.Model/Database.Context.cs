﻿//------------------------------------------------------------------------------
// <auto-generated>
//    このコードはテンプレートから生成されました。
//
//    このファイルを手動で変更すると、アプリケーションで予期しない動作が発生する可能性があります。
//    このファイルに対する手動の変更は、コードが再生成されると上書きされます。
// </auto-generated>
//------------------------------------------------------------------------------

namespace Dyxi.Muse.Model
{
    using System;
    using System.Data.Entity;
    using System.Data.Entity.Infrastructure;
    
    public partial class dyxi_museEntities : DbContext
    {
        public dyxi_museEntities()
            : base("name=dyxi_museEntities")
        {
        }
    
        protected override void OnModelCreating(DbModelBuilder modelBuilder)
        {
            throw new UnintentionalCodeFirstException();
        }
    
        public DbSet<media_contents> media_contents { get; set; }
        public DbSet<media_performers> media_performers { get; set; }
        public DbSet<media> medias { get; set; }
        public DbSet<people_members> people_members { get; set; }
        public DbSet<people> peoples { get; set; }
        public DbSet<play_logs> play_logs { get; set; }
        public DbSet<user> users { get; set; }
        public DbSet<work_composers> work_composers { get; set; }
        public DbSet<work_writers> work_writers { get; set; }
        public DbSet<work> works { get; set; }
    }
}
