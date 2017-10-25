using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Mvc.RazorPages;
using Microsoft.EntityFrameworkCore;
using ZelgaAnc.Models.Data;

namespace ZelgaAnc.Pages
{
    public class EditModel : PageModel
    {
        private readonly AppDbContext _db;

        [BindProperty]
        public Customer Customer { get; set; }

        public async Task<IActionResult> OnGetAsync(long id)
        {
            var customer = await _db.Customers.FindAsync(id);
            if (customer == null) return StatusCode(404);

            Customer = customer;
            return Page();
        }

        public async Task<IActionResult> OnPostAsync()
        {
            if (!ModelState.IsValid)
            {
                return Page();
            }

            _db.Attach(Customer).State = EntityState.Modified;

            try
            {
                await _db.SaveChangesAsync();
            }
            catch (DbUpdateConcurrencyException)
            {
                throw new InvalidOperationException($"Customer {Customer.Id} not found!");
            }

            return RedirectToPage("/Index");
        }

        public EditModel(AppDbContext db)
        {
            _db = db;
        }
    }
}
