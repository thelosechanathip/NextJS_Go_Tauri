package main

import (
	"log"
	"time"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/cors"
)

func main() {
	app := fiber.New()

	// Enable CORS for all origins (adjust in production)
	app.Use(cors.New(cors.Config{
		AllowOrigins: "http://localhost:3000, http://localhost:1420",
		AllowHeaders: "Origin, Content-Type, Accept",
	}))

	// Basic health check endpoint
	app.Get("/api/health", func(c *fiber.Ctx) error {
		return c.JSON(fiber.Map{
			"status":  "ok",
			"message": "Backend is running",
		})
	})

	// Example API endpoint
	app.Get("/api/data", func(c *fiber.Ctx) error {
		return c.JSON(fiber.Map{
			"data":      "Hello from Go Backend!",
			"timestamp": time.Now().Unix(),
		})
	})

	// Example POST endpoint
	app.Post("/api/submit", func(c *fiber.Ctx) error {
		var body map[string]interface{}
		if err := c.BodyParser(&body); err != nil {
			return c.Status(400).JSON(fiber.Map{"error": err.Error()})
		}

		return c.JSON(fiber.Map{
			"received":  body,
			"processed": true,
		})
	})

	log.Fatal(app.Listen(":8080"))
}
