using System.Security.Claims;
using Microsoft.AspNetCore.Identity;
using Microsoft.AspNetCore.Mvc;
using RoofingLeadGen.Models;

namespace RoofingLeadGen.Controllers;

[Route("account")]
public class AccountController : Controller
{
    private readonly SignInManager<ApplicationUser> _signInManager;
    private readonly UserManager<ApplicationUser> _userManager;

    public AccountController(SignInManager<ApplicationUser> signInManager, UserManager<ApplicationUser> userManager)
    {
        _signInManager = signInManager;
        _userManager = userManager;
    }

    [HttpGet("login")]
    public IActionResult Login(string? returnUrl = null)
    {
        ViewData["ReturnUrl"] = returnUrl ?? "/dashboard";
        ViewData["Error"] = TempData["Error"];
        return View();
    }

    // Register redirects to login (OAuth handles account creation automatically)
    [HttpGet("register")]
    public IActionResult Register(string? returnUrl = null)
    {
        return RedirectToAction("Login", new { returnUrl });
    }

    [HttpGet("external-login")]
    public IActionResult ExternalLogin(string provider, string? returnUrl = null)
    {
        returnUrl ??= "/dashboard";
        var redirectUrl = Url.Action("ExternalLoginCallback", "Account", new { returnUrl });
        var properties = _signInManager.ConfigureExternalAuthenticationProperties(provider, redirectUrl);
        return Challenge(properties, provider);
    }

    [HttpGet("external-login-callback")]
    public async Task<IActionResult> ExternalLoginCallback(string? returnUrl = null)
    {
        returnUrl ??= "/dashboard";

        var info = await _signInManager.GetExternalLoginInfoAsync();
        if (info == null)
        {
            TempData["Error"] = "Unable to load external login information. Please try again.";
            return RedirectToAction("Login", new { returnUrl });
        }

        // Try to sign in with this external login
        var signInResult = await _signInManager.ExternalLoginSignInAsync(
            info.LoginProvider, info.ProviderKey, isPersistent: true, bypassTwoFactor: true);

        if (signInResult.Succeeded)
        {
            return LocalRedirect(returnUrl);
        }

        // First time login - create a new account automatically
        var email = info.Principal.FindFirstValue(ClaimTypes.Email);
        var name = info.Principal.FindFirstValue(ClaimTypes.Name);

        if (string.IsNullOrEmpty(email))
        {
            TempData["Error"] = "Could not retrieve email from your account. Please try a different provider.";
            return RedirectToAction("Login", new { returnUrl });
        }

        // Check if a user with this email already exists (link the external login)
        var existingUser = await _userManager.FindByEmailAsync(email);
        if (existingUser != null)
        {
            await _userManager.AddLoginAsync(existingUser, info);
            await _signInManager.SignInAsync(existingUser, isPersistent: true);
            return LocalRedirect(returnUrl);
        }

        // Create new user
        var user = new ApplicationUser
        {
            UserName = email,
            Email = email,
            FullName = name,
            EmailConfirmed = true,
        };

        var createResult = await _userManager.CreateAsync(user);
        if (createResult.Succeeded)
        {
            await _userManager.AddLoginAsync(user, info);
            await _signInManager.SignInAsync(user, isPersistent: true);
            return LocalRedirect(returnUrl);
        }

        TempData["Error"] = "Failed to create account. Please try again.";
        return RedirectToAction("Login", new { returnUrl });
    }

    [HttpPost("logout")]
    [ValidateAntiForgeryToken]
    public async Task<IActionResult> Logout()
    {
        await _signInManager.SignOutAsync();
        return Redirect("/");
    }

    [HttpGet("logout")]
    public async Task<IActionResult> LogoutGet()
    {
        await _signInManager.SignOutAsync();
        return Redirect("/");
    }
}
